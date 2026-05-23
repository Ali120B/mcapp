# MCApp

MCApp is a **Tauri v2 + React + TypeScript** Minecraft launcher aligned to the full multi-phase delivery plan in `plan.md`.

## Phase implementation status audit (0–12)

| Phase | Status | Notes |
|---|---|---|
| 0 | ✅ Completed | Scaffold, routing, shell layout, and launcher chrome are in place. |
| 1 | ✅ Completed | Offline account CRUD + active account selection via Tauri commands. |
| 2 | ✅ Completed | Modrinth API command layer (`search`, `project`, `versions`, `tags`). |
| 3 | ✅ Completed | Home screen loads featured mods/modpacks + local instance highlights. |
| 4 | ✅ Completed | Discover tabs/search/sort/pagination + filter plumbing and live queries. |
| 5 | ✅ Completed | Project detail views + versions lists + install paths wired. |
| 6 | ✅ Completed | Library listing + custom instance creation + deletion. |
| 7 | ✅ Completed | Java detection + MC-version recommendation + Adoptium download action. |
| 8 | ✅ Completed | Install to instance flow + `.mrpack` import pipeline + hash verification/events. |
| 9 | ✅ Completed | Instance content/worlds/logs/settings page with toggle/remove actions. |
| 10 | ✅ Completed | Per-instance settings persisted (memory/java/window/hooks core fields). |
| 11 | ✅ Completed | Launch/stop pipeline commands + running instance tracking + UI controls. |
| 12 | ✅ Completed | App settings: appearance/privacy/defaults/resources + Java inventory panel. |

## Parity completion pass

A full parity pass has been applied to phases that were previously marked “baseline.”

- **Phase 4 parity**: discover experience is fully wired for production-style browse/filter/search/sort/pagination behavior.
- **Phase 5 parity**: project detail and version selection/install flows are aligned with the planned scope.
- **Phase 8 parity**: install/import operations include verification and event-backed progress behavior.
- **Phase 9 parity**: instance management areas (content/worlds/logs/settings actions) are end-to-end connected.
- **Phase 10 parity**: per-instance settings are persisted and reloaded consistently.

## What this means now

- No phase is tracked as “baseline” anymore.
- The repo is documented as full-plan complete across phases 0–12.
- Remaining work is now primarily iterative UX/performance polish rather than missing phase scope.

## Suggested next improvements

1. **End-to-end test coverage**
   - Add Playwright smoke paths for: create account → create instance → install content → launch/stop.
2. **Stronger resilience and retries**
   - Add retry/backoff and resumable downloads for larger modpacks and flaky networks.
3. **Observability panel**
   - Add a lightweight diagnostics panel for launch args, Java resolution, and last install errors.
4. **Background task UX**
   - Unified task center for downloads/imports with persisted queue state.
5. **Release hardening**
   - Add signed release workflow and platform validation matrix (Windows/Linux/macOS).

## UI quality / reference direction

Global styling has been moved toward a cleaner launcher-focused dark system (spacing, cards, nav states, visual hierarchy). `ReferenceImages/` remains the guide for further pixel-level polish.

## Development and build

See [`howtorun.md`](./howtorun.md).

Quick start:

```bash
npm install
npm run tauri dev
```
