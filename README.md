# Qobuz Windows Desktop Player

**Please note this project is in no way affiliated with Qobuz, this is merely a personal project**

This project is a Windows desktop application that acts as a web container for the Qobuz web player.

As the Qobuz desktop application wasn't available on their website, I wanted to find an alternative.

The browser apps gave me the idea, but they were missing a funcionality which I use very often, the close/minimize to tray.

And that is where this tiny application comes in, it is essentially the same as a those web applications you install on Edge/Chrome, but with the added feature of when you click on the minimize or close buttons it actually sends the app to the tray, and it being overall lighter than a full on browser in the background.

## Important Note: Do Not Use WSL

I attempted to create the project inside the WSL file system but that led to multiple issues when running commands.

It is not worth the hassle, in my opinion, for this particular app.

Just use the windows filesystem and develop on windows, it is a windows app after all.

## Key Features

- Windows desktop application
- Web container for the Qobuz web player
- Ability to minimize and close to tray
- Very lightweight

---

## Tauri + Vanilla

I have decided to use Tauri with the Vanilla rust template for this project, as it was lighter than Electron.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Project Dependencies

- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://tauri.app/)

## Onboarding (verify your environment)

Before building or developing this project, verify you have the required tools installed. The commands below are for Windows Command Line (CMD).

- Rust and Cargo (Cargo is installed alongside Rust via rustup):

  - Verify: `rustc --version` and `cargo --version`
  - Install: [Rust installer](https://www.rust-lang.org/tools/install) (installs `rustup`, which provides Cargo)

- Tauri CLI (used to build and bundle the app):

  - Verify: `tauri --version`  (or try `cargo tauri --version` / `npx tauri info`)
  - Install via Cargo: `cargo install tauri-cli`
  - Docs / downloads: [Tauri website](https://tauri.app/)

- WiX Toolset (required to create Windows installers / .msi bundles):

  - Verify: run `where candle` or `where light` in `cmd` to see if WiX binaries are on PATH.
  - Install: [WiX Toolset](https://github.com/wixtoolset/wix/releases/) — download the appropriate WiX release and follow the installer instructions. Make sure the WiX bin folder (e.g. where `candle.exe` and `light.exe` are installed) is on your PATH.
  - If not in your PATH, then add the `C:\Users\%USERNAME%\AppData\Local\tauri\WixTools<Version>` to the PATH in the Environment variables.

If any of the checks fail, follow the install links above and re-run the verification commands. Rust and Cargo are typically installed together via `rustup`; Tauri must be installed after rust due to it using Cargo; WiX is a separate Windows installer toolset required for packaging in case you want to create installation files.

## Build and launch the development profile

```bash
cargo tauri dev
```

## Build the program and create the executable binaries and installation file

```bash
cargo tauri build
```

## Windows Build Troubleshooting

If you encounter the error:

```log
error: incremental compilation: could not create session directory lock file: Incorrect function. (os error -2147024895)
```

You can fix it by disabling incremental compilation or changing the build directory:

**Disable incremental compilation:**

```cmd
set CARGO_INCREMENTAL=0
cargo build
```

**Change build directory:**

```cmd
set CARGO_TARGET_DIR=C:\temp\target
cargo build
```
