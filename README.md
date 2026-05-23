# MCApp

MCApp is a **Tauri v2 + React + TypeScript** desktop application.

## What already exists (Phase 0–6 quick check)
Based on the current codebase structure, the following core phases are represented:
- **Phase 0 (Scaffold/UI shell):** React + Vite + Tauri project scaffold is present.
- **Phase 1 (Accounts):** accounts page and related frontend API wiring exist.
- **Phase 2 (API layer):** frontend API hook (`src/hooks/api.ts`) exists.
- **Phase 3 (Home):** home page exists.
- **Phase 4 (Discover):** discover page exists.
- **Phase 5 (Project detail):** detail-oriented discover pages/assets are present in references.
- **Phase 6 (Instance library/create):** instance-related UI references and routing structure are present.

> Note: This is a high-level repository-structure check, not a full feature-completeness certification.

## Development and build
See [`howtorun.md`](./howtorun.md) for full prerequisites, local development commands, and release build targets.

Quick start:

```bash
npm install
npm run tauri dev
```
