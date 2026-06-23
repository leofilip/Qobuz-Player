# Build Error & Version Bumping

## Q1: "incremental compilation: could not create session directory lock file"

**Root cause:** You are building on the WSL (Windows Subsystem for Linux) filesystem. The Rust compiler's incremental compilation cannot create lock files reliably on the `drvfs` (WSL) mount. This is a known limitation documented in `README.md:84-107` and the project SKILL.md:175-176.

**Fixes (pick one):**

1. **Move project out of WSL** — Clone to a native Windows path (e.g. `C:\GIT_REPOS\Qobuz-Player`) and build from a Windows terminal. This is the recommended approach per the skill and README.

2. **Disable incremental compilation** (workaround):
   ```cmd
   set CARGO_INCREMENTAL=0
   cargo build
   ```

3. **Redirect build output to a Windows-native path** (workaround):
   ```cmd
   set CARGO_TARGET_DIR=C:\temp\target
   cargo build
   ```

Per the skill's known issues section, if you are inside WSL, you will hit this error. Build on the Windows filesystem only.

---

## Q2: Proper way to bump the app version

Version must match across **both** files:
- `src-tauri/Cargo.toml:3` — `version = "0.5.0"`
- `src-tauri/tauri.conf.json:4` — `"version": "0.5.0"`

The `build-menu.ps1` script handles this automatically. Either:

**Interactive:**
```
.\build-menu.ps1
```
Then select option **5** (Set version). It auto-suggests the next patch version.

**Command-line:**
```powershell
.\build-menu.ps1 v 0.5.1        # set version to 0.5.1
.\build-menu.ps1 build 0.5.1    # set version + build release
.\build-menu.ps1 dev 0.5.1      # set version + run dev mode
```

The script's `Set-VersionInternal` function (build-menu.ps1:113-197) locates the `version` line after `name = "qobuz-player"` in `Cargo.toml` and after `"productName": "qobuz-player"` in `tauri.conf.json`, then updates both. It also validates that the new version is higher than the current one using .NET `System.Version`.

The `.last-build-version` file tracks the last built version. If you try to build without bumping, the script warns you.
