# DexPet

<div align="center">
  <img src="src-tauri/icons/icon.png" alt="DexPet Icon" width="128">
</div>

> **Early development** — core pet and trick panel work; several tricks are still landing. Expect rough edges and no releases yet.
>
> A desktop pet for Windows — Dex, the folder mascot, lives on your desktop as a transparent always-on-top companion with tricks, a panel, and a tray. Offline-first and privacy-pinned: zero network unless you explicitly enable it.

---

## 📜 Table of Contents

- [✨ Features](#-features)
- [🚀 Installation](#-installation)
- [📖 Usage](#-usage)
- [⚙️ Settings](#-settings)
- [🏗️ Architecture](#-architecture)
- [🛠️ Technologies Used](#-technologies-used)
- [⚠️ Limitations](#-limitations)
- [🔒 Privacy](#-privacy)
- [📜 License](#-license)
- [📧 Contact](#-contact)

---

## ✨ Features

### The pet
- **Transparent always-on-top companion** — Dex sits on the desktop, click-through until you reach for it
- **Hit-test interactivity** — the window only becomes clickable when the cursor is actually over Dex; eye tracking follows your cursor
- **Petting & dangle** — pick Dex up and drag it around; cosmetic moods
- **Edge walking** — Dex walks the screen edges
- **Tray-owned lifetime** — closing the pet hides it; only tray Quit exits; single-instance guard

### Tricks (panel)
- **Find-my-file** — fuzzy search over a local file map
- **Open-directory** — root-checked Explorer launches
- **Reminders & focus** — local timers, break/hydration nudges
- **What-changed** — rescan diff of the watched folder
- **GitHub activity** — opt-in only; token in Windows Credential Manager, fetch runs in Rust, never in the page
- **Agent buddy** — watches local agent session *status metadata* only
- **Usage sign & streaks** — local usage stats
- **Reactive eyes/paws** — cursor, typing and gamepad reactions (event categories only, never key text)

---

## 🚀 Installation

### Prerequisites
- Windows 10/11 with WebView2 (bootstrapper included in the installer)
- Rust toolchain + Node.js — only needed to build from source

### Steps
There are no releases yet — build from source:

```sh
cd src-tauri
cargo tauri dev    # or: cargo tauri build
```

Release installers will appear on the [Releases](https://github.com/AliesterEroan/DexPet/releases) page once the app is further along.

To build from source instead:
```sh
cd src-tauri
cargo tauri build
```

---

## 📖 Usage

- **Move Dex** — grab and drag; he dangles and walks
- **Open the panel** — click Dex or the tray icon → tricks, settings, reminders
- **Hide/show** — tray menu toggles visibility without quitting
- **Quit** — tray → Quit (the only way out, by design)

---

## ⚙️ Settings

Panel → Settings covers theme/mood cosmetics, trick toggles, reminder timers, and the watched directory root (`E:\Projects` by default). GitHub activity is off until explicitly enabled and configured.

---

## 🏗️ Architecture

```
DexPet/
├── frontend/           # pet window UI (index.html/pet.css/pet.js) + trick panel (panel.html)
├── src-tauri/
│   ├── src/main.rs     # windows, tray, hit-test loop, IPC commands
│   ├── tauri.conf.json # window/tray/bundle config
│   └── icons/          # app icon (ico/png)
├── dex-light.png       # Dex mascot source art
└── make_icon.py        # icon pipeline
```

### Component Overview
- **`main.rs`** — owns both windows, the tray, the ~30 Hz cursor hit-test loop, and all privileged commands (scanner subprocess, Explorer, keychain)
- **`pet.js` / `pet.css`** — Dex's look and motion; CSS animations only, paused when hidden, `prefers-reduced-motion` respected
- **`panel.html`** — the trick panel UI

### Data Flow
1. Cursor poll (Rust) → hit-test → toggles window interactivity + feeds eye tracking
2. Trick requests → Tauri `invoke()` → Rust command → local data (scanner, timers, keychain)
3. Panel renders results; nothing leaves the machine unless GitHub activity is enabled

---

## 🛠️ Technologies Used

- **Tauri v2** — Rust backend + system webview
- **Vanilla HTML/CSS/JS** — no framework, fully offline
- **Rust** — `serde_json`, `keyring` (Credential Manager), `tauri-plugin-single-instance`
- **NSIS** — per-user Windows installer via `tauri build`

---

## ⚠️ Limitations

- **Windows only** (WebView2 dependency)
- **Early build** — core pet + trick panel working; several tricks are still landing
- **Icons8 Pulsar glyphs** in early art are licensed third-party assets — replacement or licensing required before wider distribution
- Dex is original mascot art; do not redistribute modified versions claiming authorship

---

## 🔒 Privacy

- **Offline-first** — zero network requests until GitHub activity is explicitly enabled; the gated fetch runs in Rust with a real CSP, never in the page
- **Local data only** — scanner maps, reminders, usage stats stay on disk locally
- **No content capture** — agent-buddy reads session *status* only; prompts, code and paths are never rendered, persisted or transmitted; input reactivity uses event categories/timing only, never key text
- **Credential Manager** — the optional GitHub token lives in Windows Credential Manager, never in page storage or config files

---

## 📜 License

DexPet is proprietary software — see [LICENSE](LICENSE) for the full terms.

**Copyright © 2026 AliesterEroan. All rights reserved.**

Personal, non-commercial use only. Dex is original artwork © AliesterEroan.

---

## 📧 Contact

- GitHub: [AliesterEroan/DexPet](https://github.com/AliesterEroan/DexPet)
- Issues: [Create an issue](https://github.com/AliesterEroan/DexPet/issues)

---

Built with ❤️ by AliesterEroan
