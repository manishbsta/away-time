Experimental AwayTime integration proof. These downloads are development pre-releases, not a completed screen-break app. Automatic break scheduling is not implemented yet.

- macOS: choose `aarch64` for Apple Silicon or `x86_64` for Intel. Builds use ad-hoc signing and are not Apple-notarized; macOS may require approval in Privacy & Security.
- Windows: x64 `.exe` installer. It is unsigned; Windows may show an unknown-publisher warning.
- Linux: experimental x64 AppImage, built on Ubuntu 22.04. After download, make it executable with `chmod +x AwayTime-x86_64-unknown-linux-gnu.AppImage`, then run it. Compatibility with other distributions remains unverified. X11 idle observation is implemented; Wayland idle detection, Linux sleep/wake, and focus restoration are unavailable.

Native Windows/Linux interaction, two-display previews, background/tray lifecycle, and real sleep/wake proof remain open in AT-42. Installer generation does not pass these checks. Tauri remains proposed pending the integration proof.

`SHA256SUMS.txt` contains download checksums. GitHub's source archives contain the code for this exact build, including the GPL v3 license and build instructions.
