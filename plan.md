# PulsarLauncher — Build Plan
### A privacy-first Minecraft launcher with Modrinth integration, offline accounts, and full `.mrpack` support

---

## Tech Stack Decision

### Framework: **Electron** (Node.js backend + Chromium frontend)
- **Why Electron**: It allows the entire project (backend and frontend) to be written in JavaScript/TypeScript. We can achieve similar functionality (launching Java, downloading files) using Node APIs instead of Rust or Go. It is the industry standard for lightweight-feeling desktop apps using web technologies.
- **Backend**: Node.js (Electron Main Process) for filesystem, process management, Java detection, download management, `.mrpack` parsing, and launch logic.
- **Frontend**: React + TypeScript + Vite + TailwindCSS (via `@quick-start/electron`) (Electron Renderer Process)

### Data storage
- **SQLite via `rusqlite`** (Tauri plugin `tauri-plugin-sql`) — stores instances, accounts, installed content metadata, play history.
- **JSON files** — per-instance config files (same structure as Prism/MultiMC), stored under `~/.pulsar-launcher/instances/<name>/`.

### Key Rust crates
| Purpose | Crate |
|---|---|
| HTTP (API calls, downloads) | `reqwest` (async) |
| ZIP / `.mrpack` parsing | `zip` |
| SHA1 / SHA512 verification | `sha1`, `sha2` |
| Async runtime | `tokio` |
| JSON | `serde`, `serde_json` |
| Java detection & process launch | `std::process`, `which` |
| UUID generation (offline) | `uuid` |
| Config / settings | `serde` + JSON files |

---

## Architecture Overview

```
PulsarLauncher/
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/                # Modrinth API client
│   │   │   ├── modrinth.rs     # Search, project, version, CDN endpoints
│   │   │   └── models.rs       # API response structs
│   │   ├── accounts/
│   │   │   ├── offline.rs      # Offline UUID gen, account CRUD
│   │   │   └── store.rs        # Account persistence
│   │   ├── instances/
│   │   │   ├── manager.rs      # Create, delete, duplicate, list
│   │   │   ├── install.rs      # Modpack/mod download pipeline
│   │   │   ├── mrpack.rs       # .mrpack parser & installer
│   │   │   └── launch.rs       # Java args, launch process
│   │   ├── java/
│   │   │   └── manager.rs      # Java detection, auto-download
│   │   └── settings.rs         # App-level settings
│   └── Cargo.toml
├── src/                        # React frontend
│   ├── pages/
│   │   ├── Home.tsx
│   │   ├── Discover.tsx
│   │   ├── Library.tsx
│   │   ├── InstanceDetail.tsx
│   │   └── Settings.tsx
│   ├── components/
│   │   ├── Sidebar.tsx
│   │   ├── AccountPanel.tsx
│   │   ├── ModCard.tsx
│   │   └── ...
│   └── lib/
│       ├── api.ts              # Frontend wrappers for Tauri commands
│       └── store.ts            # Zustand global state
```

---

## Phase Breakdown

Each phase produces a **fully runnable, testable build**. Phases are sized for AI-assisted implementation without overwhelming context.

---

### Phase 0 — Project Scaffolding
**Goal**: A runnable Tauri v2 app with React + Vite + Tailwind, correct window chrome (frameless titlebar, dark theme), and sidebar shell.

**Steps**:
1. `npm create tauri-app@latest pulsar-launcher -- --template react-ts`
2. Install: `tailwindcss`, `zustand`, `react-router-dom`, `lucide-react`
3. Configure Tauri: frameless window, transparent background, min size 1000×650
4. Build the shell layout:
   - Left sidebar (icons: Home, Library, Explore, Add Instance, Settings, Logout)
   - Right sidebar placeholder (account panel)
   - Main content area with router outlet
5. Set up React Router routes for all pages (empty stubs)
6. Apply base dark theme via Tailwind config (`#0d0d0d` background, `#1a1a1a` surface, `#00b16a` accent green)

**Screenshot reference**: Sidebar from `homepage.png`
**Output**: App launches, routing works, sidebar is visible

---

### Phase 1 — Account System (Offline)
**Goal**: Users can create, switch, and delete offline accounts. No Microsoft login required. Accounts persist across restarts.

**Feature set** (from `login.png`, `offlinelogin.png`):
- Right panel "Playing as" dropdown showing current account
- Dropdown expands to show: account avatar (generated blocky icon from username hash), account name, selected badge, delete button
- Three account-type icon buttons: Microsoft (grayed out / optional future), Offline, Ely.by (optional stub)
- "Add new offline account" modal: text input for player name, Login button, X close button
- Accounts stored in `~/.pulsar-launcher/accounts.json`

**Rust implementation** (`src-tauri/src/accounts/offline.rs`):
```rust
// Offline UUID = UUID v3 of "OfflinePlayer:{username}"
// This is the exact algorithm Minecraft and all major launchers use
use uuid::Uuid;

pub fn generate_offline_uuid(username: &str) -> Uuid {
    let namespace = Uuid::NAMESPACE_DNS; // placeholder; actual = custom bytes
    // Real algo: MD5 hash of bytes("OfflinePlayer:" + username), set version bits to 3
    let input = format!("OfflinePlayer:{}", username);
    let hash = md5::compute(input.as_bytes());
    let mut bytes = hash.0;
    bytes[6] = (bytes[6] & 0x0f) | 0x30; // version 3
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant bits
    Uuid::from_bytes(bytes)
}
```

**Account JSON schema**:
```json
{
  "accounts": [
    {
      "id": "uuid",
      "type": "offline",
      "username": "UltimatelyPro12",
      "uuid": "generated-uuid",
      "created_at": "timestamp"
    }
  ],
  "active_account_id": "uuid"
}
```

**Tauri commands**: `get_accounts`, `add_offline_account(username)`, `set_active_account(id)`, `delete_account(id)`

**Output**: Offline account creation, selection, and deletion fully works. UUID is deterministic per username.

---

### Phase 2 — Modrinth API Client (Rust)
**Goal**: A complete, reusable Rust module that wraps all required Modrinth v2 API endpoints. No frontend yet — just the backend service layer.

**Base URL**: `https://api.modrinth.com/v2`  
**Required User-Agent header**: `PulsarLauncher/0.1.0 (contact@yourdomain.com)` (required by Modrinth ToS)  
**Rate limit**: 300 req/min

**Endpoints to implement**:

| Endpoint | Purpose |
|---|---|
| `GET /search?query=&facets=&limit=&offset=&index=` | Search projects (mods, modpacks, resource packs, shaders, data packs) |
| `GET /project/{id\|slug}` | Project details |
| `GET /project/{id}/version` | Version list (with filters: game_version, loader) |
| `GET /version/{id}` | Single version details + file URLs |
| `GET /tag/category` | All categories |
| `GET /tag/game_version` | All MC versions |
| `GET /tag/loader` | All loaders (fabric, forge, etc.) |

**Facets format** for search filtering:
```
facets=[["project_type:modpack"],["categories:optimization"],["versions:1.21.1"]]
```
- project_type: `mod`, `modpack`, `resourcepack`, `shader`, `datapack`
- categories: `optimization`, `adventure`, `magic`, etc.

**Tauri commands to expose**:
- `search_projects(query, project_type, categories, game_version, loader, sort, limit, offset)`
- `get_project(id_or_slug)`
- `get_project_versions(project_id, game_version?, loader?)`
- `get_tags()` → categories, game versions, loaders

**Output**: All API calls work from Rust. Ready to wire to frontend in next phase.

---

### Phase 3 — Home Page
**Goal**: A fully functional home page pulling live data from Modrinth, with recently played instances.

**Layout** (from `homepage.png`):
- Top header: "Welcome back!" + breadcrumb + "No instances running" status dot + window controls
- **Jump back in** row: Last played instance card (icon, name, gamemode, last played time, Play button, ⋮ menu)
- **Discover a modpack →** row: 4 featured modpack cards (horizontal scroll) — each has: cover image, icon, name, description, download count, follower count, category tags. Clicking opens modpack detail.
- **Discover mods →** row: 4 featured mod cards (same structure)
- Right sidebar: "Playing as" account panel (Phase 1 component)
- Left sidebar: highlight Home icon

**Data sources**:
- Featured modpacks: `GET /search?facets=[["project_type:modpack"]]&limit=4&index=downloads`
- Featured mods: `GET /search?facets=[["project_type:mod"]]&limit=4&index=downloads`
- Recently played: read from local SQLite `instances` table, last_played timestamp

**React implementation**:
- `useEffect` → invoke Tauri commands on mount
- `ModpackCard` component (reusable in Discover page too)
- Loading skeleton states while API fetches

**Output**: Home page shows real Modrinth data, recently played instance visible if exists.

---

### Phase 4 — Discover Page (Browse Content)
**Goal**: Full content browsing with tabs, search, filters, and pagination — matching `discover.png`.

**Layout**:
- Tabs: Modpacks | Mods | Resource Packs | Data Packs | Shaders
- Search bar (text input, debounced 300ms)
- Sort dropdown: Relevance, Downloads, Follows, Newest, Updated
- View dropdown: 20 / 50 items per page
- Pagination: page numbers + arrows
- Result list: icon, name, author, description, tags, download count, followers, Install button
- Right sidebar filters:
  - **Categories** (checkboxes): Adventure, Challenging, Combat, etc. (loaded from `/tag/category`)
  - **Environment**: Client / Server checkboxes
  - **Game version** (collapsible, loaded from `/tag/game_version`)
  - **Loader** (collapsible): Fabric, Forge, NeoForge, Quilt
  - **License**: Open source checkbox

**Screenshot references**: `discover.png` (modpacks tab), `modsdis.png` (mods tab)

**Implementation notes**:
- Facets are built dynamically from active filter state
- Each tab change resets offset to 0 and re-fetches
- "Install" button on modpack: navigates to modpack detail → install flow
- "Add to an instance" on mods: opens instance selector modal

**Output**: Full browseable discover page with live filtering and search.

---

### Phase 5 — Project Detail Pages (Modpack + Mod)
**Goal**: Full detail view for any project — description, versions table, gallery. Matches `1779444834735_image.png`, `modpackversions.png`, `modsdetails.png`, `modsversion.png`.

**Shared layout**:
- Breadcrumb: `Discover content > [Project Name]`
- Header: icon, name, description, stats (downloads, followers), tags, Install button, ⋮ menu
- Tab bar: Description | Versions | Gallery (modpacks only)
- Right sidebar: Compatibility (MC versions), Platforms (Fabric/Forge etc.), Supported environments, Links, Creators, Details (license, published, updated)

**Description tab**: Render markdown from API `body` field using `react-markdown` + `remark-gfm`

**Versions tab** (from `modpackversions.png`, `modsversion.png`):
- Filter bar: Game versions dropdown, Channels dropdown (Release R / Beta B / Alpha A)
- Paginated table columns: Name, Game version(s), Platforms (Fabric badge), Published (relative time), Downloads
- Each row: colored version type badge (green=Release, orange=Beta, red=Alpha), download icon, external link icon
- Clicking a row or download icon: triggers install for that specific version

**Tauri commands**: `get_project(slug)`, `get_project_versions(id, filters)`

**Output**: Clicking any card in Discover or Home opens a full detail page.

---

### Phase 6 — Instance Library & Creation
**Goal**: Full instance management — view all instances, create new ones (custom or from file), manage them. Matches `instances.png`, `newinst.png`.

**Library page** (`instances.png`):
- Tab bar: All instances | Downloaded | Custom
- Search input + Sort dropdown (Name, Last played, Date created) + Group by dropdown
- Instance grid/list: icon (cube), name, loader + version badge
- Clicking an instance → instance detail page

**Create instance dialog** (`newinst.png`):
- Three creation modes (tab buttons): Custom | From File | Import From Launcher
- **Custom mode**:
  - Icon selector (click to upload custom image, Remove icon option)
  - Name text input
  - Loader selector (pill buttons): Vanilla | Fabric | Forge | NeoForge | Quilt
  - Game version dropdown (loaded from Modrinth tag API + filtered by loader support)
  - "Show all versions" toggle (shows snapshots/betas)
  - Cancel / Create buttons
- **From File mode**: file picker accepting `.mrpack` files → triggers `.mrpack` import pipeline (Phase 8)
- **Import From Launcher mode**: browse for Prism/MultiMC instance folder

**Instance data model** (SQLite):
```sql
CREATE TABLE instances (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  icon_path TEXT,
  loader TEXT NOT NULL,  -- vanilla, fabric, forge, neoforge, quilt
  game_version TEXT NOT NULL,
  minecraft_path TEXT NOT NULL,
  last_played INTEGER,
  play_time_seconds INTEGER DEFAULT 0,
  created_at INTEGER NOT NULL
);
```

**Tauri commands**: `get_instances()`, `create_instance(config)`, `delete_instance(id)`, `duplicate_instance(id)`

**Output**: Instances can be created (custom), listed, and deleted. No launching yet.

---

### Phase 7 — Java Management (Auto-detect & Auto-download)
**Goal**: Java is automatically detected or downloaded. Users never need to manually configure Java paths unless they want to.

**Detection logic** (Rust):
1. Check common locations:
   - Windows: `C:\Program Files\Java\*`, `C:\Program Files\Eclipse Adoptium\*`, `%APPDATA%\Microsoft\EdgeWebView` (no), check registry `HKLM\SOFTWARE\JavaSoft`
   - macOS: `/Library/Java/JavaVirtualMachines/*/Contents/Home`, `/usr/local/opt/openjdk*`
   - Linux: `/usr/lib/jvm/*`, `/usr/java/*`
2. Run `java -version` to get version string
3. Parse major version number from output

**Version requirements**:
- MC ≤ 1.16.5 → Java 8
- MC 1.17 → Java 16
- MC 1.18–1.20.4 → Java 17
- MC 1.20.5+ → Java 21

**Auto-download**:
- Use [Adoptium API](https://api.adoptium.net/v3/assets/latest/{major_version}/hotspot?os={os}&arch={arch}&image_type=jre) to get JRE download URL
- Download to `~/.pulsar-launcher/java/{version}/`
- Extract JDK zip/tar
- Store path in settings

**Tauri commands**: `detect_java()`, `download_java(version)`, `get_java_installations()`, `set_java_for_instance(instance_id, java_path)`

**Output**: Java is auto-configured. Instance creation validates Java availability and auto-downloads if needed.

---

### Phase 8 — Mod/Content Installation & `.mrpack` Parser
**Goal**: Install mods/modpacks from Modrinth into instances. Parse and install `.mrpack` files. Matches `installmod.png`.

**Installing a mod into an instance**:
1. User clicks "Install" on a mod in discover (filtered to instance's MC version + loader)
2. `install_content_to_instance(instance_id, project_id, version_id)` Tauri command:
   - Fetch version file info from `/v2/version/{version_id}`
   - Get primary file download URL + SHA512 hash
   - Download file to `{instance_path}/mods/filename.jar`
   - Verify SHA512 hash
   - Insert record into `instance_content` SQLite table
3. Update instance content list view

**`.mrpack` parsing and installation** (from `newinst.png` "From File"):
```rust
// mrpack = ZIP file containing:
// - modrinth.index.json (manifest)
// - overrides/ (config files, resource packs, etc.)
// - client-overrides/ (client-only files)

pub struct MrpackIndex {
    pub format_version: u32,          // always 1
    pub game: String,                 // "minecraft"
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<MrpackFile>,
    pub dependencies: HashMap<String, String>, // loader versions
}

pub struct MrpackFile {
    pub path: String,                 // e.g. "mods/sodium.jar"
    pub hashes: HashMap<String, String>, // sha1, sha512
    pub downloads: Vec<String>,       // CDN URLs
    pub file_size: u64,
}
```

**Installation pipeline**:
1. Open ZIP, parse `modrinth.index.json`
2. Read `dependencies` to determine loader (fabric/forge) + loader version + MC version
3. Create new instance with that config
4. Download all files in parallel (tokio tasks), verify hashes, place in correct paths
5. Extract `overrides/` and `client-overrides/` into instance folder
6. Show progress bar in UI (emit Tauri events per file completion)

**Tauri events emitted during install**: `install:progress { current, total, file_name }`

**Output**: Mods install from Discover page; `.mrpack` files create full instances.

---

### Phase 9 — Instance Detail Page & Content Management
**Goal**: Per-instance view showing installed mods, worlds, logs. Matches `instdeatils.png`.

**Layout**:
- Breadcrumb: `Library > WAYLAND CRAFT > Content`
- Header: icon, name, loader + version badge, play time, Play button, settings gear, ⋮ menu
- Tab bar: Content | Worlds | Logs
- **Content tab**:
  - Search bar (`Search N projects...`)
  - Bulk select checkboxes
  - "Install content" button (→ opens filtered Discover with `installmod.png` layout)
  - Refresh button
  - Table: Name (icon + name + author), Updated (version + filename), enabled toggle, delete icon, ⋮ menu
  - Each row's toggle enables/disables mod (renames `.jar` ↔ `.jar.disabled`)
- **Worlds tab**: list worlds from `saves/` folder in instance directory
- **Logs tab**: read latest log from `logs/latest.log`, auto-scroll, syntax highlighting

**"Install content" sub-page** (`installmod.png`):
- Same Discover UI but pre-filtered to instance's game version + loader
- "Back to instance" button
- Already-installed mods show "✓ Installed" badge instead of Install button
- Right sidebar: shows active filters (game version, loader) as fixed tags

**Output**: Complete per-instance mod management. Enable/disable, delete, install new content.

---

### Phase 10 — Instance Settings
**Goal**: Full per-instance settings modal. Matches `inst_settings.png`.

**Sidebar tabs**:
- **General**: Name input, icon editor, Duplicate instance button, Library groups (create group, assign instance), Delete instance (red button with confirmation)
- **Installation**: Loader version override, Game version override, mod folder path
- **Window**: Custom window resolution, fullscreen toggle, custom title
- **Java and memory**: Java executable path (auto-detected or custom), JVM arguments, min/max RAM sliders
- **Launch hooks**: Pre-launch command, wrapper command, post-exit command

**Tauri commands**: `update_instance_settings(id, settings)`, `duplicate_instance(id)`, `delete_instance(id)`, `create_library_group(name)`, `assign_instance_to_group(instance_id, group_id)`

**Output**: All instance-level settings are editable and persisted.

---

### Phase 11 — Game Launch
**Goal**: Actually launch Minecraft with the correct Java, arguments, and account.

**Launch process** (Rust `launch.rs`):
1. Load instance config
2. Resolve Java path (auto from Phase 7)
3. Download/verify Minecraft version JSON from `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`
4. Download Minecraft client JAR + all library JARs (like Prism Launcher does)
5. Download assets (asset index JSON → download all assets to shared `assets/` folder)
6. Build classpath string from all library JARs + client JAR
7. Build JVM arguments (memory, GC flags, natives path)
8. Build game arguments (username, UUID, access token `"0"` for offline, game directory, assets dir, version)
9. Spawn process: `java [jvmArgs] -cp [classpath] net.minecraft.client.main.Main [gameArgs]`
10. Capture stdout/stderr → pipe to Logs tab via Tauri events
11. Update "No instances running" / "1 instance running" status in titlebar

**Offline launch arguments** (critical):
```
--username {username}
--uuid {offline_uuid}        ← deterministic UUID from Phase 1
--accessToken 0              ← empty/zero token, accepted by offline Minecraft
--userType legacy            ← tells Minecraft not to validate with Mojang
```

**Tauri commands**: `launch_instance(id)`, `stop_instance(id)`, `get_running_instances()`

**Output**: Minecraft actually launches. Offline account works in singleplayer.

---

### Phase 12 — App Settings
**Goal**: Full application settings page. Matches `settings.png`.

**Sidebar tabs**:
- **Appearance**:
  - Color theme selector: Dark | Light | OLED | Sync with system (visual cards like in screenshot)
  - Advanced rendering toggle (blur effects)
  - Font size
- **Language**: dropdown (Beta tag)
- **Privacy**: telemetry opt-out (telemetry is OFF by default — no tracking), crash reports setting
- **Java installations**: list all detected Java installs, add custom path, set defaults per MC version range
- **Default instance options**: default loader, default game version, default RAM allocation
- **Resource management**: download concurrency slider, cache size, clear cache button

**Storage**: `~/.pulsar-launcher/settings.json` (read/write via Tauri commands)

**App version** shown at bottom left (like `Pulsar Launcher 0.10.2701` in screenshot)

**Output**: All app settings are persisted and applied globally.

---

### Phase 13 — Polish, Progress UI & Error Handling
**Goal**: Make the app feel complete and production-quality.

**Items**:
- **Global progress/download manager**: floating panel showing active downloads (like a tray), with per-file progress bars and cancel buttons
- **Toast notifications**: success/error/info toasts using a lightweight lib (`react-hot-toast`)
- **Empty states**: friendly illustrations/messages when library is empty, no search results, etc.
- **Error boundaries**: React error boundaries per page + Tauri error propagation via Result types
- **Offline mode detection**: if Modrinth API is unreachable, show cached data where available and a banner; Discover shows "Offline — showing cached results"
- **Update checker**: on startup, check GitHub releases for new launcher version (optional toast, not forced)
- **Window state persistence**: remember last window size/position
- **Keyboard shortcuts**: `Ctrl+,` for settings, `Ctrl+N` for new instance

**Output**: App feels polished and handles all edge cases gracefully.

---

## Data Directory Structure

```
~/.pulsar-launcher/
├── settings.json               # App-wide settings
├── accounts.json               # Account list + active account
├── launcher.db                 # SQLite (instances, content, history)
├── java/
│   ├── 17/                     # Auto-downloaded JRE 17
│   └── 21/                     # Auto-downloaded JRE 21
├── assets/                     # Shared Minecraft assets (all instances)
├── libraries/                  # Shared Minecraft library JARs
├── versions/
│   └── 1.21.4/                 # MC version JSON + client JAR
└── instances/
    └── WAYLAND-CRAFT/
        ├── instance.json       # Instance config
        ├── mods/               # Mod JARs
        ├── config/             # Mod configs
        ├── saves/              # Worlds
        ├── resourcepacks/
        ├── shaderpacks/
        └── logs/
```

---

## Modrinth API — No Auth Required

All content browsing (search, project details, versions, downloads) is fully public and requires no API key. Only Modrinth account features (following, rating, uploading) need auth. Since we're not implementing those, no OAuth flow is needed.

The only requirement: include a descriptive `User-Agent` header on every request:
```
User-Agent: PulsarLauncher/0.1.0 (github.com/yourname/pulsar-launcher)
```

---

## No Telemetry Policy

- No analytics, no crash reporting by default
- No pings to any third-party services
- No Modrinth account required (fully usable offline)
- Settings page has a Privacy tab making this explicit
- The only outbound requests are: Modrinth API (content browsing/download), Mojang metadata API (MC version JSON), Adoptium API (Java downloads) — all user-initiated

---

## Feature → Phase Map

| Feature | Phase |
|---|---|
| App shell + sidebar | 0 |
| Offline accounts (login.png, offlinelogin.png) | 1 |
| Modrinth API layer | 2 |
| Home / discover feed (homepage.png) | 3 |
| Discover page with filters (discover.png, modsdis.png) | 4 |
| Modpack detail + versions (1779444834735_image.png, modpackversions.png) | 5 |
| Mod detail + versions (modsdetails.png, modsversion.png) | 5 |
| Library + create instance (instances.png, newinst.png) | 6 |
| Java auto-management | 7 |
| Mod install + .mrpack import (installmod.png) | 8 |
| Instance content/worlds/logs (instdeatils.png) | 9 |
| Instance settings (inst_settings.png) | 10 |
| Minecraft game launch | 11 |
| App settings (settings.png) | 12 |
| Polish + error handling | 13 |
