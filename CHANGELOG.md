# Changelog

All notable changes to ZStartup are documented here. The project follows [Semantic Versioning](https://semver.org/).

## [0.2.0] — 2026-11-26

### Added
- **Microsoft Store / MSIX apps** (WhatsApp, Phone Link, Windows Terminal, Xbox, …) are now listed with an **App** badge and can be enabled or disabled. ([#1](https://github.com/TheHolyOneZ/ZStartup/issues/1))
- **32-bit programs** registered under `HKLM\SOFTWARE\WOW6432Node\…\Run` and `RunOnce` (e.g. iTunes Helper, Apple Mobile Device Process). ([#1](https://github.com/TheHolyOneZ/ZStartup/issues/1))
- New **Apps** filter in the sidebar.
- Every file in the startup folders is listed, not only `.lnk` shortcuts.
- Hovering the warning icon of a broken entry explains what is missing and whether it is safe to remove. ([#2](https://github.com/TheHolyOneZ/ZStartup/issues/2))

### Changed
- Enabling / disabling now uses Windows' own `StartupApproved` switch — the same one Task Manager and *Settings → Apps → Startup* use. ZStartup and Task Manager now always show the same state. Entries disabled by older ZStartup versions are still recognised and are moved back when re-enabled.
- Registry values keep their original type (`REG_EXPAND_SZ` stays `REG_EXPAND_SZ`) when toggled.
- New entries containing `%VARIABLES%` are saved as `REG_EXPAND_SZ` so Windows expands them at startup.
- Loading entries no longer blocks the window.
- Clearer error message when an action needs Administrator rights.

### Fixed
- Entries using environment variables such as `%windir%\system32\SecurityHealthSystray.exe` were wrongly marked as **Broken**. ([#2](https://github.com/TheHolyOneZ/ZStartup/issues/2))
- Unquoted paths with spaces (`C:\Program Files\…\app.exe -arg`) and bare program names (`rundll32.exe …`) were wrongly marked as **Broken**.
- Startup folder shortcuts showed a truncated target such as `C:\` or `C:\Users\` instead of the full path.
- Shortcuts without a resolvable target (e.g. MSI advertised shortcuts) are no longer flagged as broken.
- Context menu left an `Escape` key listener behind every time it was opened.

## [0.1.0] — 2026-04-09

### Added
- Initial release: HKCU / HKLM `Run` and `RunOnce` keys and user / system startup folders in one table.
- Enable / disable, delete with 5.5-second undo, add new entries.
- Search, sorting and filters by source, scope and status.
- Administrator detection with one-click restart as Administrator.
