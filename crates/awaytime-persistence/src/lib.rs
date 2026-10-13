use awaytime_core::snapshot::{CodecError, Snapshot};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

const MAX_SNAPSHOT_BYTES: u64 = 4 * 1024 * 1024;
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Storage I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Codec(#[from] CodecError),
}
#[derive(Debug)]
pub struct Loaded {
    pub snapshot: Snapshot,
    pub recovered_backup: bool,
}
/// All access is serialized with a filesystem lock. Construction and missing loads are read-only.
pub struct FileStore {
    directory: PathBuf,
}
impl FileStore {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }
    pub fn load(&self) -> Result<Option<Loaded>, StoreError> {
        if !self.directory.exists() {
            return Ok(None);
        }
        let _lock = self.lock()?;
        self.load_locked()
    }
    fn load_locked(&self) -> Result<Option<Loaded>, StoreError> {
        match read_snapshot(&self.directory.join("snapshot.json")) {
            Ok(Some(snapshot)) => Ok(Some(Loaded {
                snapshot,
                recovered_backup: false,
            })),
            Ok(None) => read_snapshot(&self.directory.join("snapshot.previous.json")).map(|s| {
                s.map(|snapshot| Loaded {
                    snapshot,
                    recovered_backup: true,
                })
            }),
            Err(StoreError::Codec(CodecError::Invalid(primary))) => {
                match read_snapshot(&self.directory.join("snapshot.previous.json"))? {
                    Some(snapshot) => Ok(Some(Loaded {
                        snapshot,
                        recovered_backup: true,
                    })),
                    None => Err(CodecError::Invalid(primary).into()),
                }
            }
            // I/O failures and incompatible schemas must never silently recover or reset.
            Err(error) => Err(error),
        }
    }
    pub fn save(&self, snapshot: &Snapshot) -> Result<(), StoreError> {
        self.save_with_hook(snapshot, || {})
    }
    fn save_with_hook(
        &self,
        snapshot: &Snapshot,
        before_commit: impl FnOnce(),
    ) -> Result<(), StoreError> {
        let bytes = snapshot.encode()?;
        if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
            return Err(CodecError::Invalid("snapshot too large".into()).into());
        }
        fs::create_dir_all(&self.directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.directory, fs::Permissions::from_mode(0o700))?;
        }
        let _lock = self.lock()?;
        let primary = self.directory.join("snapshot.json");
        match read_snapshot(&primary) {
            Ok(Some(previous)) => atomic_write(
                &self.directory,
                "snapshot.previous.json",
                &previous.encode()?,
                || {},
            )?,
            Ok(None) => {
                // Do not overwrite an incompatible backup when no primary exists.
                read_snapshot(&self.directory.join("snapshot.previous.json"))?;
            }
            Err(StoreError::Codec(CodecError::Invalid(error))) => {
                // A corrupt primary may be replaced only after a valid backup can recover it.
                if read_snapshot(&self.directory.join("snapshot.previous.json"))?.is_none() {
                    return Err(CodecError::Invalid(error).into());
                }
            }
            Err(error) => return Err(error),
        }
        atomic_write(&self.directory, "snapshot.json", &bytes, before_commit)
    }
    fn lock(&self) -> Result<File, StoreError> {
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options.open(self.directory.join(".store.lock"))?;
        lock.lock_exclusive()?;
        Ok(lock)
    }
}
impl awaytime_runtime::SnapshotStore for FileStore {
    fn load(&mut self) -> Result<Option<Snapshot>, String> {
        FileStore::load(self)
            .map(|v| v.map(|v| v.snapshot))
            .map_err(|e| e.to_string())
    }
    fn save(&mut self, snapshot: &Snapshot) -> Result<(), String> {
        FileStore::save(self, snapshot).map_err(|e| e.to_string())
    }
}
fn read_snapshot(path: &Path) -> Result<Option<Snapshot>, StoreError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = vec![];
    file.take(MAX_SNAPSHOT_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
        return Err(CodecError::Invalid("snapshot too large".into()).into());
    }
    Ok(Some(Snapshot::decode(&bytes)?))
}
fn atomic_write(
    directory: &Path,
    name: &str,
    bytes: &[u8],
    before_commit: impl FnOnce(),
) -> Result<(), StoreError> {
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    before_commit();
    temporary
        .persist(directory.join(name))
        .map_err(|e| e.error)?;
    #[cfg(unix)]
    File::open(directory)?.sync_all()?;
    // Windows replacement uses tempfile's platform implementation. Full power-loss durability
    // and directory flushing need native evidence; this is not a hardware guarantee.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn missing_load_does_not_create_a_store() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("missing");
        assert!(FileStore::new(&directory).load().unwrap().is_none());
        assert!(!directory.exists());
    }
    #[test]
    fn saved_session_and_backup_recovery_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let store = FileStore::new(temp.path());
        let mut snapshot = Snapshot::default();
        snapshot.session.work_elapsed_ms = 1234;
        snapshot.session.cycle = 2;
        snapshot.session.quota_day = Some("2026-10-13".into());
        snapshot.session.snoozes_today = 3;
        snapshot
            .extra
            .insert("optional".into(), json!([true, "keep"]));
        store.save(&snapshot).unwrap();
        assert_eq!(store.load().unwrap().unwrap().snapshot, snapshot);
        snapshot.session.work_elapsed_ms = 5678;
        store.save(&snapshot).unwrap();
        fs::write(temp.path().join("snapshot.json"), b"corrupt").unwrap();
        let recovered = store.load().unwrap().unwrap();
        assert!(recovered.recovered_backup);
        assert_eq!(recovered.snapshot.session.work_elapsed_ms, 1234);
        assert_eq!(recovered.snapshot.extra["optional"], json!([true, "keep"]));
        assert_eq!(
            fs::read(temp.path().join("snapshot.json")).unwrap(),
            b"corrupt"
        );
    }
    #[test]
    fn future_schema_cannot_recover_or_be_overwritten() {
        let temp = tempfile::tempdir().unwrap();
        let store = FileStore::new(temp.path());
        store.save(&Snapshot::default()).unwrap();
        store.save(&Snapshot::default()).unwrap();
        let primary = temp.path().join("snapshot.json");
        fs::write(&primary, br#"{"schema_version":99}"#).unwrap();
        assert!(matches!(
            store.load(),
            Err(StoreError::Codec(CodecError::Schema(99)))
        ));
        assert!(store.save(&Snapshot::default()).is_err());
        assert_eq!(fs::read(&primary).unwrap(), br#"{"schema_version":99}"#);
    }
    #[test]
    fn corrupt_files_without_backup_are_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let store = FileStore::new(temp.path());
        let primary = temp.path().join("snapshot.json");
        fs::write(&primary, b"broken").unwrap();
        assert!(store.load().is_err());
        assert!(store.save(&Snapshot::default()).is_err());
        assert_eq!(fs::read(&primary).unwrap(), b"broken");
    }
    #[test]
    fn crash_worker() {
        if let Ok(directory) = std::env::var("AWAYTIME_CRASH_TEST_DIRECTORY") {
            let store = FileStore::new(directory);
            let mut snapshot = Snapshot::default();
            snapshot.session.work_elapsed_ms = 99;
            store
                .save_with_hook(&snapshot, || std::process::exit(99))
                .unwrap();
        }
    }
    #[test]
    fn process_interruption_preserves_committed_primary() {
        let temp = tempfile::tempdir().unwrap();
        let store = FileStore::new(temp.path());
        store.save(&Snapshot::default()).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "tests::crash_worker", "--nocapture"])
            .env("AWAYTIME_CRASH_TEST_DIRECTORY", temp.path())
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(99));
        assert_eq!(
            store
                .load()
                .unwrap()
                .unwrap()
                .snapshot
                .session
                .work_elapsed_ms,
            0
        );
    }
    #[test]
    fn concurrent_writers_leave_complete_primary_and_backup() {
        let temp = tempfile::tempdir().unwrap();
        std::thread::scope(|scope| {
            for n in 1..=8 {
                let path = temp.path();
                scope.spawn(move || {
                    let mut snapshot = Snapshot::default();
                    snapshot.session.cycle = n;
                    FileStore::new(path).save(&snapshot).unwrap();
                });
            }
        });
        let current = FileStore::new(temp.path()).load().unwrap().unwrap();
        let previous = read_snapshot(&temp.path().join("snapshot.previous.json"))
            .unwrap()
            .unwrap();
        assert!((1..=8).contains(&current.snapshot.session.cycle));
        assert!((1..=8).contains(&previous.session.cycle));
        assert_ne!(current.snapshot.session.cycle, previous.session.cycle);
    }
}
