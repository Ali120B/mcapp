# MCApp

MCApp is a **Tauri v2 + React + TypeScript** Minecraft launcher work-in-progress built against the multi-phase plan in `plan.md`.

## Phase implementation status audit (0–12)

| Phase | Status | Notes |
|---|---|---|
| 0 | ✅ Implemented | Scaffold, routing, shell layout present. |
| 1 | ✅ Implemented | Offline account CRUD + active account selection via Tauri commands. |
| 2 | ✅ Implemented | Modrinth API command layer (`search`, `project`, `versions`, `tags`). |
| 3 | ✅ Implemented | Home screen loads featured mods/modpacks + local instance highlights. |
| 4 | ✅ Implemented (baseline) | Discover tabs/search/pagination present; advanced facets are partial. |
| 5 | ✅ Implemented (baseline) | Project detail + versions list present. |
| 6 | ✅ Implemented | Library listing + custom instance creation + deletion. |
| 7 | ✅ Implemented | Java detection + MC-version recommendation + Adoptium download action. |
| 8 | ✅ Implemented (baseline) | Install mod version to instance + `.mrpack` import with hash checks/events. |
| 9 | ✅ Implemented (baseline) | Instance content/worlds/logs/settings page with mod toggle/remove actions. |
| 10 | ✅ Implemented (baseline) | Per-instance settings persisted (memory/java/window/hooks core fields). |
| 11 | ✅ Implemented in this update | Launch/stop pipeline commands + running instance tracking + UI controls. |
| 12 | ✅ Implemented in this update | Application settings page: appearance/privacy/defaults/resources + Java list. |

> “Baseline” means implemented end-to-end with currently practical scope in this repo, with room for deeper parity enhancements.

## What was added now (Phases 11 and 12)

### Phase 11 — Launch flow
- Added backend commands:
  - `launch_instance(instance_id)`
  - `stop_instance(instance_id)`
  - `get_running_instances()`
- Launch currently executes configured Java with memory args and process tracking, stores run metadata, and emits launch log events.
- Added instance detail UI controls for **Play/Stop** with running status indicator.

### Phase 12 — App settings
- Added a dedicated **Settings** page and navigation route.
- Added persisted app settings (local storage) for:
  - Appearance presets (dark/light/oled/system)
  - Blur toggle and font size
  - Privacy toggles (telemetry/crash reports)
  - Default instance options (loader/version/RAM)
  - Resource options (concurrency/cache + clear cache)
- Added Java installations section in Settings using backend detection.

## UI quality / reference direction
- Updated global styling to a cleaner launcher-like dark visual system with improved spacing, gradients, cards, active nav states, and less “unstyled/demo” feeling.
- Reference images in `ReferenceImages/` remain the style target for future pixel-level parity passes.

## Development and build
See [`howtorun.md`](./howtorun.md).

Quick start:

```bash
npm install
npm run tauri dev
```
