# Forma

**Forma is a Windows desktop wallpaper engine built around real-time cellular automata.**

It runs as a real app (installer + tray + startup support), can attach behind desktop icons, and provides a control window for tuning simulation settings.

## Download

- Latest release page: https://github.com/Fatin-Ishraq/Forma/releases/latest
- Direct Windows installer (`.exe`): https://github.com/Fatin-Ishraq/Forma/releases/download/v0.1.0/FormaWallpaper-Setup-0.1.0.exe

## Demo

- Live web demo: https://fatin-ishraq.github.io/Forma/

## Preview

![Forma running live](www/forma-live.png)

## Windows-First Features

- Native Windows app with installer/uninstaller
- Tray app workflow (`Wallpaper Enabled`, `Open Controls Window`, `Launch at Startup`)
- WorkerW wallpaper embedding with clean fallback mode
- Persistent app settings in `%APPDATA%/Forma/config.json`
- Startup integration via HKCU Run
- Logging + diagnostics export for support/recovery
- Recovery handling for sleep/resume and Explorer host changes

## User Flow (Windows)

1. Download and run `FormaWallpaper-Setup-0.1.0.exe`
2. Launch Forma (or enable launch at startup)
3. Use tray menu to toggle wallpaper mode
4. Open controls window to adjust mode/rules/theme/speed
5. Click `SET WALLPAPER` to apply settings to wallpaper runtime

## Requirements

- Windows 10/11 (x64)
- Microsoft Edge WebView2 Runtime (usually already present on modern Windows)

## Build From Source

### Prerequisites

- Rust (stable)
- `wasm-pack`
- Inno Setup 6 (`ISCC.exe`) for installer builds

### 1) Build web assets

```bash
wasm-pack build --target web --out-dir www/pkg
```

### 2) Run desktop app

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

## Project Layout

```text
.
├── src/                                  # Rust simulation core (WASM)
├── www/                                  # Shared web/control UI
├── desktop/forma-wallpaper/              # Windows desktop host app
│   ├── src/windows_app/                  # host, tray, wallpaper embedding, startup, logs
│   ├── installer/FormaWallpaper.iss      # Inno Setup script
│   └── scripts/                          # build/release helper scripts
└── README.md
```

## Notes

- If wallpaper attach fails, Forma automatically falls back to normal window mode.
- User config is intentionally preserved across upgrade/uninstall.
- Logs are written to `%LOCALAPPDATA%/Forma/logs`.
