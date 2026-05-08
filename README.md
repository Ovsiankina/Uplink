<h1 align="center">
  <img src="ui/extra/images/logo.png" width=200 height=200/><br>
  Uplink
</h1>

<h4 align="center">Privacy First, Modular, P2P messaging client built atop Warp.</h4>

<br/>

Uplink is written in pure Rust with a UI in [Dioxus](https://github.com/DioxusLabs) (which is also written in Rust). It was developed as a new foundation for implementing Warp features in a universal application.

The goal should be to build a hyper-customizable application that can run anywhere and support extensions.

![Uplink UI](https://i.imgur.com/X4AGeLz.png)

---
## Pre-Compiled Development

For rapid inspection of our deployed binaries, you can open the settings once signed into Uplink, then navigate to `About` and click the version number 10 times, enabling a `Developer` section in the settings. From here, you can enable experimental features and helpful dev tools.


## Quickstart

To get running fast, ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

**Standard Run:**
```
cargo run --bin uplink
```

**Rapid Release Testing:**
This version will run close to release but without recompiling crates every time.
```
cargo run --bin uplink --profile=rapid
```

---


## Dependency List

**macOS M1+**
| Dep  | Install Command                                                  |
|------|------------------------------------------------------------------|
| Build Tools| `xcode-select --install` |
| Homebrew | `/bin/bash -c "\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"` |
| Rust | `curl --proto  '=https' --tlsv1.2 -sSf https://sh.rustup.rs` | sh |
| cmake | `brew install cmake` |
| ffmpeg | `brew install ffmpeg` |
| audio opus | `brew install opus` |

You can also run [macos-install_dependencies.sh](https://github.com/Satellite-im/Uplink/blob/sara/add-macos-script/macos-install_dependencies.sh) to install all of the above in bulk.

**Windows 10+**
| Dep  | Install Command                                                  |
|------|------------------------------------------------------------------|
| Rust | [Installation Guide](https://www.rust-lang.org/tools/install) |
| ffmpeg | [Installation Guide](https://www.geeksforgeeks.org/how-to-install-ffmpeg-on-windows/) |


**Ubuntu WSL (Maybe also Ubuntu + Debian)**
| Dep  | Install Command                                                  |
|------|------------------------------------------------------------------|
| Rust | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Build Essentials | `sudo apt install build-essential` |
| pkg-config | `sudo apt-get install pkg-config` |
| alsa-sys | `sudo apt install librust-alsa-sys-dev` |
| libgtk-dev | `sudo apt-get install libgtk-3-dev` |
| libsoup-dev | `sudo apt install libsoup-3.0-dev` |
| Tauri Deps | `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev` |
| ffmpeg| `sudo apt-get install ffmpeg` |
| libopus-dev| `sudo apt-get install libopus-dev` |
| libxdo-dev| `sudo apt install libxdo-dev` |

**Fedora 38**
| Dep  | Install Command                                                  |
|------|------------------------------------------------------------------|
| Rust | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Build Essentials | `sudo dnf groupinstall "Development Tools" "Development Libraries"` |
| pkg-config | `sudo dnf install pkg-config` |
| alsa libs & headers | `sudo dnf install alsa-lib-devel` |
| libgtk-dev | `sudo dnf install gtk3-devel` |
| libsoup-dev | `sudo dnf install libsoup3-devel` |
| Tauri Deps | `sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget librsvg2-devel libindicator-devel` |
| ffmpeg| `sudo dnf install ffmpeg` |
| libopus-dev| `sudo dnf install opus-devel` |
| libxdo-dev| `sudo dnf install libxdo-devel` |


## Testing

The project follows a layered testing strategy on the `message` module: unit and integration tests in Rust (run via `cargo test`), plus an automated and a manual E2E test for the full message-sending flow.

### Unit tests

Unit tests live inline in `kit/src/components/message/mod.rs` under `#[cfg(test)] mod tests`. They cover the pure text-processing functions (`format_text`, `markdown`, `replace_emojis`, `wrap_links_with_a_tags`, `is_only_emojis`, `process_string`, `stack_processor`) and the mention pipeline. The mention tests use `State::mock()` (Fake pattern) to isolate the Warp dependency.

```bash
cargo test -p kit --lib components::message
```

### Integration tests

Integration tests live in `kit/tests/integration_message.rs`. By Rust convention, files in `tests/` are compiled as a separate crate and only have access to the public API — this forces the tests to use the module the way a real consumer would.

```bash
cargo test -p kit --test integration_message
```

Or run unit + integration tests in one go:

```bash
cargo test
```

### Manual E2E test

Scenario: send a message containing bold text, a link, and an emoji, then verify that all formatting is applied on the receiving side.

| Step | Action | Expected |
|------|--------|----------|
| 1 | Launch Uplink and log in | The app opens, the user is logged in |
| 2 | Open an existing conversation | The conversation displays with its history |
| 3 | Type `**Bonjour** ! Regarde https://example.com :)` in the input | The text appears in the input field |
| 4 | Send the message | The message appears in the conversation |
| 5 | Inspect the rendering | "Bonjour" is bold, the link is clickable, `:)` is replaced by 🙂 |

### Automated E2E test — Windows only

The automated E2E test is in `tests/` (a separate TypeScript project using **WebdriverIO 8 + Appium 2 + WinAppDriver**). It launches two Uplink instances, creates two accounts, sends friend requests between them, and exercises the message input (send, empty, max length, emoji, paste, etc.).

**One-time prerequisites:**

1. **Node.js v22** (v24 has known compatibility issues with this project)
2. **Rust** with a built Uplink debug binary at `target/debug/uplink.exe` (`cargo build --bin uplink`)
3. **WinAppDriver** installed at `C:\Program Files\Windows Application Driver\WinAppDriver.exe` ([download](https://github.com/microsoft/WinAppDriver/releases))
4. **Windows Developer Mode** enabled (Settings → Privacy & Security → For developers → Developer Mode → ON). Required for WinAppDriver to initialize.

**Run:**

```powershell
cd tests
npm install     # one-time
.\msg-test.ps1
```

The script frees ports 4723 / 4724, kills any leftover `uplink.exe`, then launches WebdriverIO with the Windows-chats config. Uplink and Appium are started automatically by `@wdio/appium-service` — do not start them manually.

Logs are written to `tests/appium.log` for debugging.

---

## Contributing

All contributions are welcome! Please keep in mind we're still a relatively small team, and any work done to ensure contributions don't cause bugs or issues in the application is much appreciated.

Guidelines for contributing are located in the [`contributing_process.md`](docs/contributing_process.md).

# Contributors

![GitHub Contributors Image](https://contrib.rocks/image?repo=Satellite-im/Uplink)

tag
