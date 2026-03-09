# Forma

Realtime cellular automata as a native Windows wallpaper.

[![Release](https://img.shields.io/github/v/release/Fatin-Ishraq/Forma)](https://github.com/Fatin-Ishraq/Forma/releases/latest)
[![Pages](https://github.com/Fatin-Ishraq/Forma/actions/workflows/deploy-pages.yml/badge.svg)](https://github.com/Fatin-Ishraq/Forma/actions/workflows/deploy-pages.yml)
[![Live Demo](https://img.shields.io/badge/demo-live-2ea44f)](https://fatin-ishraq.github.io/Forma/)

Forma is a Windows desktop wallpaper engine built around a Rust + WebAssembly simulation core.  
It can run behind desktop icons through WorkerW embedding, exposes a controls window, and supports tray-first operation (toggle wallpaper, startup, diagnostics, updates).

## Preview

![Forma running live](www/forma-preview.gif)

## Quick Start (Windows)

1. Download the latest installer: <https://github.com/Fatin-Ishraq/Forma/releases/latest>
2. Run the `FormaWallpaper-Setup-*.exe` installer from release assets.
3. Launch Forma from Start menu.
4. Use the tray icon to toggle `Wallpaper Active` and open `Control Window`.
5. Adjust mode/rules/theme/performance and click `SET WALLPAPER`.

## Live Demo

Try the browser build: <https://fatin-ishraq.github.io/Forma/>

## Feature Highlights

- Native Windows desktop app with installer/uninstaller.
- WorkerW wallpaper attach with fallback to normal window mode if attach fails.
- Real-time CA engine (Conway + Generations) compiled to WASM.
- Presets for both families (`Life`, `HighLife`, `Brian's Brain`, `StarWars`, etc.).
- Tray controls for resolution, FPS cap, theme, interaction profile, startup, diagnostics.
- Automatic recovery handling for Explorer host resets and suspend/resume.
- Persistent settings in `%APPDATA%/Forma/config.json`.
- Structured logs in `%LOCALAPPDATA%/Forma/logs`.

## Controls At A Glance

- Rule modes: `Conway`, `Generations`
- Themes: `Lab`, `Ember`, `Bio`, `Mono`
- Tray resolutions: `512`, `768`, `1024`
- Tray FPS caps: `30`, `60`, `120`
- Interaction profiles: `Subtle`, `Balanced`, `Expressive`
- Generations states: `2..20`

## Requirements

- Windows 10/11 (x64)
- Microsoft Edge WebView2 Runtime (normally preinstalled on modern Windows)

## Build From Source

### Prerequisites

- Rust (stable)
- `wasm-pack`
- Inno Setup 6 (`ISCC.exe`) for installer builds

### 1) Build WASM package

```bash
wasm-pack build --target web --out-dir www/pkg
```

### 2) Run desktop host

```bash
cargo run --manifest-path desktop/forma-wallpaper/Cargo.toml
```

### 3) Build release binary

```bash
cargo build --release --manifest-path desktop/forma-wallpaper/Cargo.toml
```

### 4) Build installer

```powershell
powershell -ExecutionPolicy Bypass -File desktop/forma-wallpaper/scripts/build-installer.ps1
```

### 5) Generate release checksum

```powershell
powershell -ExecutionPolicy Bypass -File desktop/forma-wallpaper/scripts/release-checksum.ps1
```

## Troubleshooting

- Wallpaper not attaching:
  Toggle `Wallpaper Active` from tray once, then reopen controls.
- Explorer restarted / desktop changed:
  Forma auto-rebinds the wallpaper host, but tray-toggle can force a clean reattach.
- Need support data:
  Use tray `Export Diagnostics...` and share the generated file.

## Project Layout

```text
.
├── src/                                  # Rust simulation core (WASM)
├── www/                                  # Web UI + control surface
├── desktop/forma-wallpaper/              # Native Windows host app
│   ├── src/windows_app/                  # tray, startup, logging, wallpaper host, IPC
│   ├── installer/FormaWallpaper.iss      # Inno Setup script
│   └── scripts/                          # release/build helpers
└── README.md
```

## Contributing

- Open an issue for bugs, crashes, or performance regressions.
- For UI/behavior changes, include before/after screenshots or a short clip.
- For desktop host fixes, include relevant logs from `%LOCALAPPDATA%/Forma/logs`.

## License

No license file is currently included in this repository.
