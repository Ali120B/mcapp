# How to Run MCApp

This repository is a **Tauri v2 + React + TypeScript** desktop app.

## 1) Prerequisites
Install these first:
- **Node.js 20 LTS** (recommended)
- **npm** (comes with Node.js)
- **Rust toolchain** (stable) via `rustup`
- **Tauri native prerequisites** for your OS:
  - Linux: GTK/WebKit2GTK/build-essential deps
  - macOS: Xcode Command Line Tools
  - Windows: MSVC Build Tools + WebView2 runtime

Official Tauri prerequisites: <https://v2.tauri.app/start/prerequisites/>

---

## 2) Install dependencies
From repo root:

```bash
npm install
```

This installs JS dependencies and creates/updates `package-lock.json`.

---

## 3) Run in development
### Web-only dev (Vite)
```bash
npm run dev
```

### Desktop dev (Tauri + frontend, one command)
```bash
npm run run
```

This uses the `run` script (`tauri dev`), which automatically triggers `beforeDevCommand` (`npm run dev`) from `src-tauri/tauri.conf.json` so both the web UI and desktop app start together.

If you prefer, you can still run:
```bash
npm run tauri dev
```

---

## 4) Compile / build
### Frontend build only
```bash
npm run build
```

### Desktop release build (host platform)
```bash
npm run tauri build
```

Artifacts are emitted under `src-tauri/target/release/bundle/`.

---

## 5) Cross-platform release targets requested
You asked for:
- **Windows x64**
- **Linux AppImage**
- **macOS Intel x64**

Use the following commands:

### Windows x64
```bash
npm run tauri build -- --target x86_64-pc-windows-msvc
```

### Linux x64 (AppImage)
```bash
npm run tauri build -- --target x86_64-unknown-linux-gnu --bundles appimage
```

### macOS Intel x64
```bash
npm run tauri build -- --target x86_64-apple-darwin
```

---

## 6) Important CI/release note (cross-compiling)
Cross-compiling Tauri desktop bundles across OSes is non-trivial. The most reliable setup is:
- build Windows artifacts on a Windows runner,
- Linux AppImage on a Linux runner,
- macOS Intel on a macOS runner.

Recommended approach: use a GitHub Actions matrix with one runner per OS/target.

---

## 7) Quick command summary
```bash
# install deps
npm install

# dev
npm run dev
npm run run
# optional equivalent
npm run tauri dev

# build
npm run build
npm run tauri build

# target-specific
npm run tauri build -- --target x86_64-pc-windows-msvc
npm run tauri build -- --target x86_64-unknown-linux-gnu --bundles appimage
npm run tauri build -- --target x86_64-apple-darwin
```
