# Contributing to AwayTime

Branch names and commit messages must follow the conventions below strictly.
Rename a noncompliant branch and correct noncompliant commit messages before
requesting review. Reviewers must request corrections before merging.

## Branch workflow

Create work branches from `develop` and open pull requests targeting `develop`.
Release maintainers promote reviewed changes from `develop` to `main` through
a release pull request. Do not target `main` with feature or bug-fix pull requests.

## Branch names

Use this format:

```text
<type>/<short-description>
```

Use lowercase letters, numbers, and hyphens in the description. Separate words
with hyphens. Keep the description short but specific. Do not use spaces,
underscores, personal names, or vague descriptions such as `updates` or `stuff`.

| Type       | Use for                                      | Example                            |
| ---------- | -------------------------------------------- | ---------------------------------- |
| `feat`     | New user-facing behavior                     | `feat/automatic-break-scheduling`  |
| `fix`      | Correcting a bug                             | `fix/tray-menu-resume`             |
| `docs`     | Documentation                                | `docs/macos-installation`          |
| `refactor` | Restructuring code without changing behavior | `refactor/session-command-gateway` |
| `test`     | Adding or improving tests                    | `test/snapshot-recovery`           |
| `perf`     | Improving performance                        | `perf/reduce-idle-polling`         |
| `build`    | Dependencies or build tooling                | `build/update-tauri-cli`           |
| `ci`       | GitHub workflows or automation               | `ci/add-code-quality-checks`       |
| `chore`    | Other maintenance                            | `chore/configure-git-hooks`        |

If a branch addresses a tracked issue, include its identifier in lowercase:
`fix/at-42-snapshot-recovery`. Use the same naming format for branches created
by automation; do not add a tool-specific prefix.
The shared branches `main` and `develop` are exceptions to this naming format.

Valid names:

```text
feat/break-preview-shortcut
fix/restore-session-after-restart
ci/check-rust-formatting
```

Invalid names:

```text
new-feature
fix/My_Bug
chore/stuff
```

## Commit messages

Use a consistent description of each change (Conventional Commits):

```text
<type>(<optional-scope>): <description>
```

Use a type from the table above. A scope identifies the affected part of the app,
such as `ui`, `core`, `runtime`, `storage`, `desktop`, or `deps`.
Omit the parentheses when there is no scope.

Follow these rules strictly:

- Use a lowercase type and scope, followed by a colon and one space.
- Start the description with a lowercase action verb: `add`, `fix`, `remove`,
  `update`, or `prevent`. Describe the result rather than saying `changes`.
- Use the imperative form: `add break reminders`, not `added break reminders`.
- Keep the first line at most 72 characters and omit a trailing period.
- Keep each commit focused on one coherent change. The commit type should
  describe that change; it does not have to match the branch type.
- When explanation is needed, leave a blank line after the first line and explain
  why the change is needed in the body. Include issue references there.
- Never use messages such as `wip`, `fix`, `updates`, or `final changes`.

Examples:

```text
feat(ui): add break preview shortcut
fix(storage): preserve the backup during snapshot recovery
docs: explain macOS installation
refactor(core): extract pause decision logic
test(runtime): cover repeated pause commands
build(deps): update the Tauri CLI
ci: check Rust formatting on pull requests
chore: configure Husky hooks
```

A commit with a body:

```text
fix(storage): preserve the backup during snapshot recovery

Keep the last valid snapshot available if writing the recovered data fails.

Refs: AT-42
```

For an incompatible change, add `!` before the colon and a `BREAKING CHANGE:`
footer explaining the impact and migration. For example:

```text
feat(storage)!: require versioned snapshot files

BREAKING CHANGE: unversioned snapshots are no longer accepted. Convert existing
files to the versioned format before upgrading.
```

PR titles must follow the same commit-message format so squash merges produce
compliant commits. When creating a release merge commit, edit its message to
follow the convention, for example `chore(release): promote develop to main`.

## Checks and enforcement

Run `npm ci` to install the local Husky hooks. The pre-commit hook checks staged
frontend files and Rust formatting. The pre-push hook checks TypeScript.
GitHub's Code quality workflow checks frontend lint, formatting, TypeScript,
and Rust formatting.

Branch naming and commit-message conventions are mandatory project policy.
The current hooks and workflow do not automatically validate these names or
messages; contributors and reviewers must check them. Do not skip required
checks to get a change merged.
