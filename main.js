const API_BASE = 'https://api.modrinth.com/v2';

const store = {
  page: 'home',
  projectType: 'modpack',
  searchQuery: '',
  discoverPage: 1,
  pageSize: 10,
  sort: 'downloads',
  selectedProject: null,
  projectVersions: [],
  tags: { categories: [], loaders: [], gameVersions: [] },
  featured: { modpacks: [], mods: [] },
  discoverResults: { hits: [], total_hits: 0 },
  loading: false,
  error: '',
  accounts: JSON.parse(localStorage.accounts || '[]'),
  activeAccount: localStorage.activeAccount || '',
  instances: JSON.parse(localStorage.instances || '[]'),
};

const q = (s) => document.querySelector(s);
const save = () => {
  localStorage.accounts = JSON.stringify(store.accounts);
  localStorage.activeAccount = store.activeAccount;
  localStorage.instances = JSON.stringify(store.instances);
};
const uid = () => crypto.randomUUID();

async function md5Hex(input) {
  const data = new TextEncoder().encode(input);
  const hash = await crypto.subtle.digest('MD5', data).catch(() => null);
  if (!hash) return '00000000000000000000000000000000';
  return Array.from(new Uint8Array(hash)).map((b) => b.toString(16).padStart(2, '0')).join('');
}

async function offlineUuid(username) {
  const input = `OfflinePlayer:${username}`;
  const hex = await md5Hex(input);
  const bytes = hex.match(/.{1,2}/g).map((h) => parseInt(h, 16));
  bytes[6] = (bytes[6] & 0x0f) | 0x30;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const out = bytes.map((b) => b.toString(16).padStart(2, '0')).join('');
  return `${out.slice(0, 8)}-${out.slice(8, 12)}-${out.slice(12, 16)}-${out.slice(16, 20)}-${out.slice(20)}`;
}

async function modrinth(path, params = {}) {
  const url = new URL(`${API_BASE}${path}`);
  Object.entries(params).forEach(([k, v]) => {
    if (v !== undefined && v !== null && v !== '') url.searchParams.set(k, String(v));
  });
  const res = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      'X-Launcher': 'PulsarLauncher-Web-Prototype',
    },
  });
  if (!res.ok) throw new Error(`Modrinth ${res.status}`);
  return res.json();
}

async function loadTags() {
  const [categories, loaders, gameVersions] = await Promise.all([
    modrinth('/tag/category'),
    modrinth('/tag/loader'),
    modrinth('/tag/game_version'),
  ]);
  store.tags.categories = categories.map((x) => x.name).slice(0, 24);
  store.tags.loaders = loaders.map((x) => x.name);
  store.tags.gameVersions = gameVersions.map((x) => x.version).slice(0, 20);
}

async function loadHome() {
  const [modpacks, mods] = await Promise.all([
    modrinth('/search', { limit: 4, index: 'downloads', facets: JSON.stringify([[`project_type:modpack`]]) }),
    modrinth('/search', { limit: 4, index: 'downloads', facets: JSON.stringify([[`project_type:mod`]]) }),
  ]);
  store.featured.modpacks = modpacks.hits;
  store.featured.mods = mods.hits;
}

async function searchDiscover() {
  const offset = (store.discoverPage - 1) * store.pageSize;
  const facets = [[`project_type:${store.projectType}`]];
  const data = await modrinth('/search', {
    query: store.searchQuery,
    limit: store.pageSize,
    offset,
    index: store.sort,
    facets: JSON.stringify(facets),
  });
  store.discoverResults = data;
}

async function openProject(slug) {
  store.loading = true;
  render();
  try {
    const project = await modrinth(`/project/${slug}`);
    const versions = await modrinth(`/project/${project.id}/version`);
    store.selectedProject = project;
    store.projectVersions = versions;
    store.page = 'project';
  } catch (e) { store.error = String(e); }
  store.loading = false;
  render();
}

function nav() {
  return ['home', 'discover', 'library', 'settings'].map((p) => `<div class="navbtn ${store.page === p ? 'active' : ''}" data-nav="${p}">${p[0].toUpperCase()}</div>`).join('');
}

function accountPanel() {
  const a = store.accounts.find((x) => x.id === store.activeAccount);
  return `<div class=panel><h3>Playing as</h3><div>${a ? a.username : 'None selected'}</div><small>${a ? a.uuid : ''}</small><div class=row style="margin-top:8px"><button class=primary id=addacc>Offline</button></div></div>
  <div class=panel><b>Accounts</b><div class=list>${store.accounts.map((x) => `<div class=item><div>${x.username}<div class=tag>${x.type}</div></div><div><button data-use="${x.id}">Use</button> <button data-delacc="${x.id}">X</button></div></div>`).join('') || '<small>No accounts</small>'}</div></div>`;
}

const projectCard = (p) => `<div class=card><span class=badge>${p.project_type}</span><h4>${p.title}</h4><p>${p.description || ''}</p><div class=tag>${p.downloads?.toLocaleString?.() || 0} downloads</div><div class=row><button class=primary data-open="${p.slug}">Details</button></div></div>`;

function home() {
  return `<div class=top><h2>Welcome back!</h2><span class=status>${store.instances.length ? 'Ready to play' : 'No instances running'}</span></div>
  <h3>Jump back in</h3><div class=cards>${store.instances.slice(0, 3).map((i) => `<div class=card><h4>${i.name}</h4><div class=tag>${i.loader} • ${i.version}</div></div>`).join('') || '<div class=card>No instances yet</div>'}</div>
  <h3>Discover a modpack</h3><div class=cards>${store.featured.modpacks.map(projectCard).join('')}</div>
  <h3>Discover mods</h3><div class=cards>${store.featured.mods.map(projectCard).join('')}</div>`;
}

function discover() {
  const maxPage = Math.max(1, Math.ceil((store.discoverResults.total_hits || 0) / store.pageSize));
  return `<div class=top><h2>Discover content</h2><div class=row>
  <input id=search class=input placeholder="Search" value="${store.searchQuery}">
  <select id=ptype><option value="modpack" ${store.projectType === 'modpack' ? 'selected' : ''}>modpack</option><option value="mod" ${store.projectType === 'mod' ? 'selected' : ''}>mod</option><option value="resourcepack" ${store.projectType === 'resourcepack' ? 'selected' : ''}>resourcepack</option><option value="shader" ${store.projectType === 'shader' ? 'selected' : ''}>shader</option></select>
  <select id=sort><option value="downloads" ${store.sort === 'downloads' ? 'selected' : ''}>downloads</option><option value="follows" ${store.sort === 'follows' ? 'selected' : ''}>follows</option><option value="updated" ${store.sort === 'updated' ? 'selected' : ''}>updated</option></select>
  </div></div>
  <div class=hsplit><div><div class=list>${(store.discoverResults.hits || []).map((h) => `<div class=item><div><b>${h.title}</b><div class=tag>${h.author} • ${h.description || ''}</div></div><button data-open="${h.slug}">Open</button></div>`).join('')}</div><div class=row><button id=prev ${store.discoverPage <= 1 ? 'disabled' : ''}>Prev</button><span>Page ${store.discoverPage}/${maxPage}</span><button id=next ${store.discoverPage >= maxPage ? 'disabled' : ''}>Next</button></div></div>
  <aside class=panel><b>Tags</b><small>Categories: ${store.tags.categories.slice(0, 8).join(', ')}</small><br><small>Loaders: ${store.tags.loaders.join(', ')}</small><br><small>MC Versions: ${store.tags.gameVersions.slice(0, 6).join(', ')}</small></aside></div>`;
}

function projectDetail() {
  const p = store.selectedProject;
  if (!p) return '<h3>No project selected</h3>';
  return `<div class=top><h2>${p.title}</h2><button id=backdisc>Back</button></div>
  <div class=panel><p>${p.description || ''}</p><small>${p.project_type} • ${p.downloads?.toLocaleString?.() || 0} downloads • ${p.followers || 0} followers</small></div>
  <h3>Versions</h3><div class=list>${store.projectVersions.slice(0, 25).map((v) => `<div class=item><div><b>${v.name}</b><div class=tag>${v.version_number} • ${v.loaders.join(', ')} • ${v.game_versions.join(', ')}</div></div><button data-install="${p.title} ${v.version_number}">Install</button></div>`).join('')}</div>`;
}

function library() { return `<h2>Instances</h2><div class=list>${store.instances.map((i) => `<div class=item><div><b>${i.name}</b><div class=tag>${i.loader} • ${i.version}</div></div></div>`).join('') || 'Empty'}</div>`; }
function settings() { return '<h2>Settings</h2><div class=panel>Global settings placeholder (Phase 12)</div>'; }

const pageView = () => ({ home, discover, library, settings, project: projectDetail }[store.page] || home)();

function render() {
  document.getElementById('app').innerHTML = `<div class=app><aside class=sidebar><div class=logo>PL</div>${nav()}</aside><main class=main>${store.error ? `<div class=panel>${store.error}</div>` : ''}${store.loading ? '<div class=panel>Loading...</div>' : pageView()}</main><aside class=right>${accountPanel()}</aside></div>`;
  bind();
}

function bind() {
  document.querySelectorAll('[data-nav]').forEach((n) => n.onclick = async () => { store.page = n.dataset.nav; if (store.page === 'discover') await searchDiscover(); render(); });
  q('#addacc')?.addEventListener('click', async () => {
    const u = prompt('Offline username');
    if (!u) return;
    const id = uid();
    const uuid = await offlineUuid(u);
    store.accounts.push({ id, type: 'offline', username: u, uuid });
    store.activeAccount = id;
    save();
    render();
  });
  document.querySelectorAll('[data-use]').forEach((b) => b.onclick = () => { store.activeAccount = b.dataset.use; save(); render(); });
  document.querySelectorAll('[data-delacc]').forEach((b) => b.onclick = () => { store.accounts = store.accounts.filter((a) => a.id !== b.dataset.delacc); if (store.activeAccount === b.dataset.delacc) store.activeAccount = ''; save(); render(); });
  q('#search')?.addEventListener('change', async (e) => { store.searchQuery = e.target.value; store.discoverPage = 1; await searchDiscover(); render(); });
  q('#ptype')?.addEventListener('change', async (e) => { store.projectType = e.target.value; store.discoverPage = 1; await searchDiscover(); render(); });
  q('#sort')?.addEventListener('change', async (e) => { store.sort = e.target.value; store.discoverPage = 1; await searchDiscover(); render(); });
  q('#prev')?.addEventListener('click', async () => { store.discoverPage = Math.max(1, store.discoverPage - 1); await searchDiscover(); render(); });
  q('#next')?.addEventListener('click', async () => { store.discoverPage += 1; await searchDiscover(); render(); });
  document.querySelectorAll('[data-open]').forEach((b) => b.onclick = () => openProject(b.dataset.open));
  q('#backdisc')?.addEventListener('click', async () => { store.page = 'discover'; await searchDiscover(); render(); });
}

(async function init() {
  try {
    store.loading = true;
    render();
    await loadTags();
    await loadHome();
    await searchDiscover();
    store.loading = false;
    render();
  } catch (e) {
    store.loading = false;
    store.error = `Failed to initialize launcher data: ${String(e)}`;
    render();
  }
})();
