# Qobuz Player

This project is a Windows desktop application that acts as a web container for the Qobuz web player. In addition to providing access to the Qobuz web interface, it includes features such as minimizing and closing the application to the system tray for improved user experience and convenience.

## Important Note: Do Not Use WSL

Do not use Windows Subsystem for Linux (WSL) to build or run this project. Use the native Windows environment for all development and execution to avoid compatibility issues and runtime errors.

## Key Features

- Windows desktop application
- Web container for the Qobuz web player
- Ability to minimize to tray
- Ability to close to tray

---

## Tauri + Vanilla

This template should help get you started developing with Tauri in vanilla HTML, CSS and Javascript.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Project Dependencies

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started.html)
- [Tauri CLI](https://tauri.app/)
- Node.js (for Tauri frontend, if applicable)

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
