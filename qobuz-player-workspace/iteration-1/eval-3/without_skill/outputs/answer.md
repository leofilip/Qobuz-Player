# Build Error: Session Directory Lock File

## Cause

The error `incremental compilation: could not create session directory lock file: Incorrect function (os error -2147024895)` is caused by running the Rust/Tauri build from inside the WSL (Windows Subsystem for Linux) filesystem. Windows filesystem operations (locking, symlinks) don't translate correctly across the WSL layer, and Rust's incremental compilation relies on file locking that fails in this environment.

As noted in `README.md:84-91`:

> It is possible you may be running the project on the WSL file system, if so my recomendation would be to place it outside of WSL.

## Fixes

**Recommended:** Move the project to the native Windows filesystem (e.g., `C:\GIT_REPOS\Qobuz-Player`) and build from there instead.

**Workarounds** (from `README.md:93-106`):

1. **Disable incremental compilation** (slower builds):
   ```cmd
   set CARGO_INCREMENTAL=0
   cargo build
   ```

2. **Change the build directory** to a Windows-native path:
   ```cmd
   set CARGO_TARGET_DIR=C:\temp\target
   cargo build
   ```

---

# Proper Way to Bump Version for a Release Build

The app version must be kept in sync across **two files**:

| File | Current Value |
|---|---|
| `src-tauri/Cargo.toml:3` | `version = "0.5.0"` |
| `src-tauri/tauri.conf.json:4` | `"version": "0.5.0"` |

## Method 1: Use the build script (recommended)

The PowerShell script `build-menu.ps1` has a **Set Version** option (option 5) that automates this:

**Interactive mode:**
```powershell
.\build-menu.ps1
# Select option 5, then enter e.g. 0.5.1
```

**Command-line mode:**
```powershell
.\build-menu.ps1 v 0.5.1       # just set the version
.\build-menu.ps1 build 0.5.1   # set version AND build release
.\build-menu.ps1 dev 0.5.1     # set version AND run dev mode
```

The script (`build-menu.ps1:113-197`) performs the following:
1. Validates the version format is `MAJOR.MINOR.PATCH`
2. Checks the new version is higher than the current version
3. Updates `version = "X.Y.Z"` in `src-tauri/Cargo.toml` (locating it right after `name = "qobuz-player"`)
4. Updates `"version": "X.Y.Z"` in `src-tauri/tauri.conf.json` (locating it right after `"productName": "qobuz-player"`)

It also tracks the last build version in `.last-build-version` and will warn if you try to build without bumping.

## Method 2: Manually

Edit both files to the same version string:

- `src-tauri/Cargo.toml`: line 3 — change `version = "0.5.0"` to `version = "0.5.1"`
- `src-tauri/tauri.conf.json`: line 4 — change `"version": "0.5.0"` to `"version": "0.5.1"`
