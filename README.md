# Hus-Clean

**Hus-Clean** is a lightweight, secure tool for cleaning up temporary files and unwanted cookies/cache (ads, trackers) on Windows. This version is a complete rewrite in **Rust**, designed to be more robust, lighter and more privacy-respecting than the original C++/Qt version.

---

## 📌 Features

- **Temporary file cleanup** (`%TEMP%`, `%LOCALAPPDATA%\Temp`), **recursive** scan (subfolders included).
- **Removal of unwanted cookies** (Chrome, Firefox, Brave, Edge, Opera, Vivaldi), with detection of **multiple profiles** per browser.
- **Browser cache cleanup**, in addition to cookies.
- **Sent to the recycle bin** rather than permanently deleted: can be undone.
- **Individual file selection** before cleanup (checkbox per item).
- **Intuitive interface** with confirmation before any deletion.
- **No administrator rights required** (no UAC prompt).
- **No network connection**: zero telemetry, by design.
- **Full transparency**: no file is deleted without explicit confirmation.

---

## 📥 Installation

### Requirements
- Windows 10/11.

### Download
Simply grab **`hus-clean.exe`** at the root of this repository and place it wherever you want on your computer. It's a single, standalone executable: no archive to unpack, no folder of `.dll` files alongside it.

See **`Procédure Hus-Clean.pdf`** for a step-by-step illustrated guide (download, unblocking on Windows, usage).

---

## 🚀 Usage

1. Launch `hus-clean.exe`.
2. Click **"Scan"** and choose what you want to analyze (temporary files / cookies / cache).
3. Uncheck the items you want to keep.
4. Click **"Clean"** and confirm.
5. The selected items are sent to the recycle bin (except those locked by an open browser or a system file currently in use, which are clearly flagged).

---

## ⚠️ Note

- Close your browsers before cleaning their cookies/cache.
- Since the software isn't digitally signed (code signing is paid), Windows may show a warning on first launch ("came from another computer" block + SmartScreen): see the PDF guide to unblock it — this is normal for any free, unsigned software.

---

## 🛠️ Building from source

The source code is in the [`src/`](src/) folder. It's a Rust project (Cargo workspace):

- `src/crates/hus-clean-core`: scanning and cleaning logic, no GUI dependency, independently testable (`cargo test`).
- `src/crates/hus-clean-gui`: graphical interface ([egui](https://github.com/emilk/egui)/eframe).
- `src/docker`: Dockerfiles to reproduce the cross-compilation environment (Linux → Windows, via `mingw-w64` and `cargo-xwin`/MSVC) and the test environment.

Build (from `src/`, with the Rust toolchain + `x86_64-pc-windows-msvc` target installed, see `src/docker/Dockerfile`):

```
cargo xwin build --release --target x86_64-pc-windows-msvc
```

---

## Security

- No `unsafe` in the project's code (verified via `cargo-geiger`).
- Dependencies audited via `cargo-audit` (RustSec vulnerability database).
- Static analysis via `cargo clippy`.
- The binary only links against standard Windows system DLLs, no external dependency to install.

---

© 2026 Spellskite-coding (https://github.com/Spellskite-coding) & Marwane Toury (https://github.com/MarwaneToury).
