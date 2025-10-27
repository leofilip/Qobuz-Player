# Qobuz Windows Desktop Player

This is a simple web container app, based on Tauri2 and taking advantage of Webview2, in order to allow the user to be able to have the Qobuz Web Player as a windows desktop app with the closing and minimizing functions having been changed to minimize and close to the tray.

## Disclaimer

Please note this project is in no way affiliated with Qobuz, this is merely a personal project.

As the Qobuz desktop application wasn't available on their website, I wanted to find an alternative.

The browser apps gave me the idea, but they were missing a funcionality which I use very often, the close/minimize to tray.

And that is where this tiny application comes in, it is essentially the same as a those web applications you install on Edge/Chrome, but with the added feature of when you click on the minimize or close buttons it actually sends the app to the tray, and it being overall lighter than a full-on browser in the background.

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

## OS informnation

This app was designed and built on Windows 11 and not tested on other versions, so I can't guarantee it will work fine with older versions of Windows.

## Project Dependencies

- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://tauri.app/)

## Onboarding

I have included a neat batch script that can be run on the terminal and has a menu to help the user get started more easily.

```cmd
.\build-menu.cmd
```

It features the following options:

- Check for missing dependencies - this is helpful to check if there the user has all the necessary tools installed (rust, tauri, etc)
- Run the app in development mode - this runs the cargo tauri dev command, which builds and opens the app in debug mode for quick testing
- Build the app in release mode - this runs the cargo tauri build command, which performs the final build and generates the msi installation file
- Open installer folder - this opens the folder where the generated .msi files get stored in the project
- Set Version - helper for updating the version of the app across the relevant files, it requires the format be number.number.number (major.minor.patch)

If any of the checks fail on the first command, follow the install links above and re-run the verification commands.
Rust and Cargo are typically installed together via `rustup`.
Tauri must be installed after rust due to it using Cargo.
WiX is a separate Windows installer toolset required for packaging the build into the .msi installation files.

## Versioning and Releases

For each new release, update the `version` field in both `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` to keep them in sync. This ensures your build and installer metadata match the intended release version.

Example:

- In `src-tauri/Cargo.toml`:
  version = "0.1.2"
- In `src-tauri/tauri.conf.json`:
  "version": "0.1.2"

Update both files before building or publishing a new release. You can use the build-menu script's "Set version" option to automate this step.

## Windows Build Troubleshooting

If you encounter the error:

```log
error: incremental compilation: could not create session directory lock file: Incorrect function. (os error -2147024895)
```

It is possible you may be running the project on the WSL file system, if so my recomendation would be to place it outside of WSL.

You can also "fix" the error by disabling incremental compilation or changing the build directory:

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
