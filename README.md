# PowerModeTray

[![CI](https://github.com/9enki/power-mode-tray/actions/workflows/ci.yml/badge.svg)](https://github.com/9enki/power-mode-tray/actions/workflows/ci.yml)

A tiny tray app that cycles the Windows 11 power mode (Settings > System > Power & battery > Power mode)
with a single click on the tray icon or a hotkey.

- **Left click / hotkey**: Best power efficiency → Balanced → Best performance → Best power efficiency …
- **Right click**: pick a mode directly, see the current hotkey, or quit
- The icon is a gauge from the standard Windows icon font (Segoe Fluent Icons). The needle position shows the mode: left for efficiency, center for balanced, right for performance
- Follows changes made elsewhere, such as the Settings app, switching between AC and battery, or changing the light/dark theme
- While Windows Energy Saver is active, no click, hotkey, or menu item changes the mode, matching the Settings app. The icon becomes a battery with a leaf, and both the tooltip and the menu explain why
- A single executable of about 140 KB written in Rust against the Win32 API only. No GUI framework, no extra runtime, and about 2 MB of private memory while resident
- No administrator rights, and no writes to the registry or disk

## Usage

```
PowerModeTray.exe [--hotkey <key>]
```

| Argument | Meaning |
| --- | --- |
| (none) | The hotkey is `Ctrl+Alt+P` |
| `--hotkey Ctrl+Shift+F12` | Change the hotkey. Modifiers are `Ctrl` / `Alt` / `Shift` / `Win`, and keys include `A-Z` / `0-9` / `F1-F24` (`F1-F24` also work on their own) |
| `--hotkey none` | Run without a hotkey |
| `--help` | Show this help |

- Each press advances the mode by one, exactly like a left click
- If another app already owns the key, a notification appears and the app keeps running without a hotkey
- A second instance exits silently. To change arguments, quit from the right-click menu first, then start it again
- Only the current power source (plugged in or on battery) is changed, matching the Settings app
- Nothing changes while Energy Saver is active. Turning it off restores normal behavior. The state comes from both a power setting notification and `GetSystemPowerStatus`

### Keeping the icon out of the overflow menu

On first launch the icon lands in the taskbar overflow, behind the `^` button. To show it permanently,
turn PowerModeTray on under Settings > Personalization > Taskbar > Other system tray icons
(`Win + R` then `ms-settings:taskbar` opens that page directly).
Windows stores this per executable path, so moving the file means setting it again.

## Installation

### From source

Building needs Rust ([rustup](https://rustup.rs/), `stable-x86_64-pc-windows-msvc`) and the
"Desktop development with C++" workload of the Visual Studio Build Tools, which provides the linker and the Windows SDK.

```powershell
# From WSL
powershell.exe -NoProfile -ExecutionPolicy Bypass -File install.ps1

# From PowerShell on Windows
.\install.ps1
```

`install.ps1` builds the app, stops any running instance, copies the executable over
`%LOCALAPPDATA%\Programs\PowerModeTray\`, and starts it. The same command also updates an existing install.
Because the destination never changes, the tray visibility setting survives updates.

| Option | Meaning |
| --- | --- |
| `-Hotkey Ctrl+Shift+F9` | Value passed as `--hotkey`. Defaults to the previous startup setting, or the built-in default |
| `-Startup` | Place a shortcut in the Startup folder so the app launches at sign-in. Not done by default |
| `-NoBuild` | Skip the build and deploy the existing `bin\PowerModeTray.exe` |
| `-NoLaunch` | Deploy without starting the app |
| `-Uninstall` | Stop the app and remove the install folder and the Startup shortcut |

### winget

The release executable is published through [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs)
(available once the package is accepted).

```powershell
winget install 9enki.PowerModeTray
winget upgrade 9enki.PowerModeTray
```

## About side effects

- Nothing is written to the registry, to disk, or to any setting. Switching the mode is the same API call the Settings app makes
- No administrator rights. The manifest declares `asInvoker`
- Only the current power source (AC or battery) is touched. The other one is left alone
- The hotkey is a `RegisterHotKey` registration and is released when the app exits
- The Energy Saver state is a `RegisterPowerSettingNotification` subscription and is unsubscribed when the app exits
- No extra runtime and no GUI framework. Only DLLs that ship with Windows are loaded
- A mutex prevents a second instance

Measured while resident on real hardware (Windows 11, 150% DPI):

| Metric | Value |
| --- | --- |
| Private memory | about 1.7 MB |
| Working set | about 10 MB |
| CPU while idle | too small to measure. It reads the state once every 2 seconds |
| Threads / handles | 6 / about 140 |

Switching modes also leaves all 3611 power-related registry values unchanged, writes nothing to AppData,
and registers no startup entry.

## How it works

The power mode in the Settings app is an "overlay power scheme", readable and writable through
`PowerSetActiveOverlayScheme` and `PowerGetEffectiveOverlayScheme` in `powrprof.dll`. This app just calls them.

| Mode | GUID |
| --- | --- |
| Best power efficiency | `961cc777-2547-4f9d-8174-7d86181b8a7a` |
| Balanced | `00000000-0000-0000-0000-000000000000` |
| Best performance | `ded574b5-45a0-4f42-8737-46345c09c238` |

## Development

| Command | Purpose |
| --- | --- |
| `.\build.ps1` | Runs `cargo build --release` and copies the result to `bin\PowerModeTray.exe`. The version comes from `version` in `Cargo.toml` |
| `.\test.ps1` | Runs `cargo test`, covering hotkey and argument parsing plus Energy Saver icon selection |

`cargo build --release` and `cargo test` work directly too. From WSL, run
`powershell.exe -NoProfile -ExecutionPolicy Bypass -File <script>`
(builds on a `\\wsl.localhost` path are slow, so the scripts point `CARGO_TARGET_DIR` at a Windows temp folder).
Save `.ps1` files as UTF-8 with a BOM, since they contain Japanese and Windows PowerShell 5.1 reads BOM-less files as Shift-JIS.

There are two dependencies: `windows-sys` for the raw Win32 bindings, and `embed-resource` at build time
for the version resource and the application manifest.

## Layout

| Path | Contents |
| --- | --- |
| `src/main.rs` | Window, message loop, menu, and state |
| `src/tray.rs` | Icon rendering (icon-font glyphs drawn with GDI) and Shell_NotifyIcon |
| `src/power.rs` | Reading and writing the power mode via powrprof.dll, plus the Energy Saver state |
| `src/hotkey.rs` / `src/cli.rs` | Hotkey notation and command-line parsing, with unit tests |
| `build.rs` / `app.manifest` | Embeds the version resource and the manifest declaring no elevation and per-monitor DPI awareness |
| `Cargo.toml` | Dependencies and version |
| `build.ps1` / `test.ps1` / `install.ps1` | Build, test, and personal install |
| `scripts/common.ps1` | Helpers shared by the scripts |
| `winget/` / `msix/` | Generation of the distribution packages (winget manifests, MSIX) |
| `.github/workflows/` | CI and release automation |

## License

MIT License
