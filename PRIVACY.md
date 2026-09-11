# Privacy Policy

**PowerModeTray does not collect, store, or transmit any personal data.**

Last updated: 2026-09-11

## What the app does

PowerModeTray reads and changes the Windows power mode on the device it runs on. To do so it uses
documented and undocumented Windows APIs (`powrprof.dll` overlay power scheme functions,
`GetSystemPowerStatus`, and a power setting notification subscription) and reads one registry value
to follow the light/dark theme.

If you turn on "Start with Windows" from the tray menu, the app writes a single value named
`PowerModeTray` under the per-user key
`HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`, holding the path to the
executable. Turning the option off deletes that value. Nothing else is ever written.

## Data collection

None. Specifically, the app:

- Collects no personal information, telemetry, analytics, crash reports, or usage statistics
- Makes no network connections of any kind
- Writes nothing to disk and creates no configuration files, logs, or caches
- Writes nothing to the registry, apart from the opt-in "Start with Windows" value described above
- Contains no advertising and no third-party SDKs

The only state the app changes is the Windows power mode itself, which is the purpose of the app
and is identical to what the Windows Settings app does, plus the "Start with Windows" value when
you choose to enable it.

## Third parties

No data is shared with anyone, because no data is collected.

## Children

The app collects no data from anyone, including children.

## Changes

Any change to this policy will be published in this file in the project repository.

## Contact

Questions about this policy can be raised as an issue at
https://github.com/9enki/power-mode-tray/issues
