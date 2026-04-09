<div align="center">
  <img src="src-tauri/icons/128x128.png" width="96" height="96" alt="ZStartup logo" />

  <h1>ZStartup</h1>
  <p><strong>A clean, fast Windows startup manager — built with Tauri + React.</strong></p>

  <p>
    <img src="https://img.shields.io/badge/version-0.1.0-7c6af7?style=flat-square" alt="version" />
    <img src="https://img.shields.io/badge/platform-Windows%2010%20%2F%2011-0078d4?style=flat-square&logo=windows" alt="platform" />
    <img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24c8db?style=flat-square&logo=tauri" alt="tauri" />
    <img src="https://img.shields.io/badge/license-MIT-22c55e?style=flat-square" alt="license" />
  </p>

  <br />

  <br /><br />

  <a href="../../releases/latest/download/ZStartup_0.1.0_x64_en-US.msi">
    <img src="https://img.shields.io/badge/Download-.msi%20Installer-7c6af7?style=for-the-badge&logo=windows&logoColor=white" alt="Download MSI" />
  </a>
  &nbsp;
  <a href="../../releases/latest/download/ZStartup_0.1.0_x64-setup.exe">
    <img src="https://img.shields.io/badge/Download-NSIS%20Setup-4f46e5?style=for-the-badge&logo=windows&logoColor=white" alt="Download NSIS" />
  </a>

</div>

---

## What is ZStartup?

ZStartup gives you full control over every program that launches at Windows startup — registry entries and startup folders, for both the current user and all users. No bloat, no telemetry, no bullshit.

Everything runs inside a clean dark UI with instant search, sorting, and a safe-delete undo system so you don't accidentally nuke something important.

---

## Features

| | Feature | Description |
|---|---|---|
| 🗄️ | **Full Registry Control** | Manages `HKCU` and `HKLM` Run / RunOnce keys in one unified table |
| 📁 | **Startup Folder Support** | User + system startup folders displayed alongside registry entries |
| 🔍 | **Instant Search & Filter** | Filter by source, scope, status (enabled / disabled / broken) |
| 🗑️ | **Safe Delete with Undo** | 5.5-second undo window via toast notification before the entry is removed |
| 🛡️ | **Admin Detection** | Auto-detects elevation level, one-click restart as Administrator |
| ⚡ | **Lightweight** | Tauri-based — no Electron, no Node runtime, tiny binary |

---

## Screenshots

> _Full pixel-accurate UI preview available on the [landing page](website/index.html)._

The app renders every startup entry in a sortable table with toggle switches, source/scope badges, and per-row action buttons. Broken entries (file doesn't exist on disk) are highlighted with an amber warning indicator.

---

## Installation

### Option A — MSI Installer *(recommended)*
1. Download `ZStartup_0.1.0_x64_en-US.msi`
2. Run the installer — no extra dependencies needed
3. Launch ZStartup from the Start Menu

### Option B — NSIS Setup
1. Download `ZStartup_0.1.0_x64-setup.exe`
2. Run the setup wizard
3. Done

> **Note:** For full functionality (system-level startup entries), run ZStartup as Administrator.

---

## Usage

1. **Launch the app** — all startup entries load automatically on open
2. **Toggle entries** — click the switch on any row to enable or disable
3. **Delete entries** — click the trash icon; you have 5.5 seconds to undo via the toast
4. **Filter** — use the sidebar to filter by registry, folder, enabled, disabled, broken, user, or system scope
5. **Search** — type in the search bar to filter by name, path, or command
6. **Add entry** — click **+ Add** to register a new startup program
7. **Refresh** — the app auto-refreshes on window focus, or click the refresh button manually

---

## Building from Source

**Requirements:**
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (stable)
- [Tauri CLI v2](https://tauri.app/start/prerequisites/)

```bash
# Clone the repo
git clone https://github.com/TheHolyOneZ/ZStartup.git
cd ZStartup

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build release binary
npm run tauri build
```

Built artifacts will be in `src-tauri/target/release/bundle/`.

---

## Tech Stack

| Layer | Technology |
|---|---|
| UI Framework | React 18 + TypeScript |
| Desktop Shell | Tauri v2 (Rust) |
| State Management | Zustand |
| Bundler | Vite |
| Icons | Lucide React |
| Styling | Pure CSS variables (no framework) |

---

## Project Structure

```
ZStartup/
├── src/                    # React frontend
│   ├── components/
│   │   ├── entries/        # EntryTable, EntryRow, AddModal
│   │   ├── layout/         # Titlebar, Sidebar, StatusBar
│   │   └── ui/             # Toast, ContextMenu, AdminBanner
│   ├── store/              # Zustand stores (startupStore, uiStore)
│   ├── lib/                # Tauri command bindings
│   └── styles/             # Global CSS + design tokens
├── src-tauri/              # Rust backend
│   ├── src/
│   │   └── startup/        # Registry + folder parsing logic
│   ├── icons/              # App icons (ICO, ICNS, PNG)
│   └── capabilities/       # Tauri permission config
└── website/                # Landing page (single index.html)
    ├── icons/
    └── releases/
```

---

## License

MIT © 2026 [TheHolyOneZ](https://github.com/TheHolyOneZ)
