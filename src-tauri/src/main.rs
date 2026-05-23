use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::{Path, PathBuf}, process::Command};
use tauri::{AppHandle, Emitter};

const USER_AGENT_VALUE: &str = "mcapp-launcher/0.1.0 (hello@example.com)";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OfflineAccount { id: String, username: String, kind: String }
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct AccountsState { active_account_id: Option<String>, accounts: Vec<OfflineAccount> }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct InstanceSummary { id: String, name: String, mc_version: String, loader: String, icon: Option<String>, last_played: Option<String>, group: Option<String> }
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct InstancesState { instances: Vec<InstanceSummary> }

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct InstanceModEntry { file_name: String, project_id: String, version_id: String, enabled: bool, path: String }
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct InstanceSettings { memory_mb: u32, java_path: Option<String>, width: u32, height: u32, pre_launch_hook: Option<String>, post_exit_hook: Option<String> }
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct InstanceState { mods: Vec<InstanceModEntry>, worlds: Vec<String>, logs: Vec<String>, settings: InstanceSettings }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModrinthSearchResponse { hits: Vec<ModrinthHit>, total_hits: Option<u64> }
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModrinthHit { project_id: String, title: String, description: String, icon_url: Option<String>, downloads: u64, project_type: String, slug: String, author: Option<String>, categories: Option<Vec<String>> }
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModrinthProject { id: String, slug: Option<String>, title: String, description: String, body: Option<String>, project_type: String, icon_url: Option<String>, downloads: Option<u64>, followers: Option<u64>, categories: Option<Vec<String>> }
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModrinthVersion { id: String, name: String, version_number: String, game_versions: Vec<String>, loaders: Vec<String>, date_published: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JavaInstallation { path: String, version: String, major: u32 }
#[derive(Debug, Serialize, Deserialize, Clone)]
struct JavaRecommendation { mc_version: String, required_major: u32, installed_match: Option<JavaInstallation> }

#[derive(Debug, Serialize, Deserialize)]
struct VersionFile { url: String, filename: String, hashes: HashMap<String, String> }
#[derive(Debug, Serialize, Deserialize)]
struct MrpackManifest { files: Vec<MrpackFile>, overrides: String }
#[derive(Debug, Serialize, Deserialize)]
struct MrpackFile { path: String, hashes: HashMap<String, String>, downloads: Vec<String> }

fn app_file(app: &AppHandle, name: &str) -> Result<PathBuf, String> { let dir = app.path().app_data_dir().map_err(|e| e.to_string())?; fs::create_dir_all(&dir).map_err(|e| e.to_string())?; Ok(dir.join(name)) }
fn load_json<T: for<'de> Deserialize<'de> + Default>(path: PathBuf) -> Result<T, String> { if !path.exists() { return Ok(T::default()); } serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string()) }
fn save_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> { fs::write(path, serde_json::to_string_pretty(value).map_err(|e| e.to_string())?).map_err(|e| e.to_string()) }
fn sha1_hex(bytes: &[u8]) -> String { use sha1::Digest; let mut h = sha1::Sha1::new(); h.update(bytes); format!("{:x}", h.finalize()) }

fn load_accounts(app: &AppHandle) -> Result<AccountsState, String> { load_json(app_file(app, "accounts.json")?) }
fn save_accounts(app: &AppHandle, state: &AccountsState) -> Result<(), String> { save_json(app_file(app, "accounts.json")?, state) }
fn load_instances(app: &AppHandle) -> Result<InstancesState, String> { load_json(app_file(app, "instances.json")?) }
fn save_instances(app: &AppHandle, state: &InstancesState) -> Result<(), String> { save_json(app_file(app, "instances.json")?, state) }

fn instance_dir(app: &AppHandle, instance_id: &str) -> Result<PathBuf, String> { Ok(app_file(app, "instances")?.join(instance_id)) }
fn load_instance_state(app: &AppHandle, instance_id: &str) -> Result<InstanceState, String> { let p = instance_dir(app, instance_id)?.join("instance_state.json"); load_json(p) }
fn save_instance_state(app: &AppHandle, instance_id: &str, state: &InstanceState) -> Result<(), String> { let dir = instance_dir(app, instance_id)?; fs::create_dir_all(&dir).map_err(|e| e.to_string())?; save_json(dir.join("instance_state.json"), state) }

fn offline_uuid(name: &str) -> String { let digest = md5::compute(format!("OfflinePlayer:{name}").as_bytes()); let mut b = digest.0; b[6] = (b[6] & 0x0f) | 0x30; b[8] = (b[8] & 0x3f) | 0x80; format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", u32::from_be_bytes([b[0],b[1],b[2],b[3]]), u16::from_be_bytes([b[4],b[5]]), u16::from_be_bytes([b[6],b[7]]), u16::from_be_bytes([b[8],b[9]]), u64::from_be_bytes([0,0,b[10],b[11],b[12],b[13],b[14],b[15]])) }
fn modrinth_client() -> Result<reqwest::Client, String> { let mut h = HeaderMap::new(); h.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE)); reqwest::Client::builder().default_headers(h).build().map_err(|e| e.to_string()) }

fn parse_major(version: &str) -> Option<u32> { if let Some(stripped) = version.split('"').nth(1) { let token = stripped.split('.').next()?; return token.parse::<u32>().ok(); } None }
fn candidate_java_paths() -> Vec<String> { let mut out = vec!["java".to_string()]; if let Ok(home) = std::env::var("JAVA_HOME") { out.push(format!("{home}/bin/java")); } if cfg!(target_os = "windows") { out.extend(["C:/Program Files/Java/jdk-21/bin/java.exe".into(), "C:/Program Files/Eclipse Adoptium/jdk-21/bin/java.exe".into()]); } else if cfg!(target_os = "macos") { out.extend(["/usr/bin/java".into(), "/Library/Java/JavaVirtualMachines/temurin-21.jdk/Contents/Home/bin/java".into()]); } else { out.extend(["/usr/bin/java".into(), "/usr/lib/jvm/java-21-openjdk-amd64/bin/java".into(), "/usr/lib/jvm/temurin-21-jdk-amd64/bin/java".into()]); } out }
fn probe_java(path: &str) -> Option<JavaInstallation> { let output = Command::new(path).arg("-version").output().ok()?; let text = String::from_utf8_lossy(&output.stderr).to_string() + &String::from_utf8_lossy(&output.stdout); let first = text.lines().next()?.to_string(); let major = parse_major(&first)?; Some(JavaInstallation { path: path.to_string(), version: first, major }) }
fn required_java_for_mc(mc: &str) -> u32 { let minor = mc.split('.').nth(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(21); if minor >= 20 { 21 } else if minor >= 18 { 17 } else { 8 } }

#[tauri::command] fn get_accounts(app: AppHandle) -> Result<AccountsState, String> { load_accounts(&app) }
#[tauri::command] fn add_offline_account(app: AppHandle, username: String) -> Result<AccountsState, String> { let mut s = load_accounts(&app)?; let id = offline_uuid(&username); if !s.accounts.iter().any(|a| a.id==id) { s.accounts.push(OfflineAccount{id:id.clone(), username, kind:"offline".into()}); } s.active_account_id=Some(id); save_accounts(&app,&s)?; Ok(s) }
#[tauri::command] fn delete_account(app: AppHandle, account_id: String) -> Result<AccountsState, String> { let mut s = load_accounts(&app)?; s.accounts.retain(|a| a.id!=account_id); if s.active_account_id.as_deref()==Some(&account_id){ s.active_account_id=s.accounts.first().map(|a|a.id.clone()); } save_accounts(&app,&s)?; Ok(s) }
#[tauri::command] fn set_active_account(app: AppHandle, account_id: String) -> Result<AccountsState, String> { let mut s = load_accounts(&app)?; if s.accounts.iter().any(|a|a.id==account_id){ s.active_account_id=Some(account_id); save_accounts(&app,&s)?;} Ok(s) }

#[tauri::command] fn list_instances(app: AppHandle) -> Result<InstancesState, String> { load_instances(&app) }
#[tauri::command] fn create_instance(app: AppHandle, name: String, mc_version: String, loader: String) -> Result<InstancesState, String> { let mut s=load_instances(&app)?; let id = format!("{}-{}", name.to_lowercase().replace(' ',"-"), s.instances.len()+1); s.instances.push(InstanceSummary{id:id.clone(),name,mc_version,loader,icon:None,last_played:None,group:None}); save_instances(&app,&s)?; save_instance_state(&app, &id, &InstanceState { settings: InstanceSettings { memory_mb: 4096, width: 1280, height: 720, ..Default::default() }, ..Default::default() })?; Ok(s)}
#[tauri::command] fn delete_instance(app: AppHandle, instance_id: String) -> Result<InstancesState, String> { let mut s=load_instances(&app)?; s.instances.retain(|i|i.id!=instance_id); save_instances(&app,&s)?; let _ = fs::remove_dir_all(instance_dir(&app, &instance_id)?); Ok(s)}

#[tauri::command] fn get_instance_state(app: AppHandle, instance_id: String) -> Result<InstanceState, String> { load_instance_state(&app, &instance_id) }
#[tauri::command] fn set_instance_settings(app: AppHandle, instance_id: String, settings: InstanceSettings) -> Result<InstanceState, String> { let mut st = load_instance_state(&app, &instance_id)?; st.settings = settings; save_instance_state(&app, &instance_id, &st)?; Ok(st) }
#[tauri::command] fn toggle_instance_mod(app: AppHandle, instance_id: String, file_name: String, enabled: bool) -> Result<InstanceState, String> { let mut st = load_instance_state(&app, &instance_id)?; if let Some(m) = st.mods.iter_mut().find(|m| m.file_name==file_name) { m.enabled = enabled; } save_instance_state(&app, &instance_id, &st)?; Ok(st) }
#[tauri::command] fn remove_instance_mod(app: AppHandle, instance_id: String, file_name: String) -> Result<InstanceState, String> { let mut st = load_instance_state(&app, &instance_id)?; st.mods.retain(|m| m.file_name!=file_name); save_instance_state(&app, &instance_id, &st)?; Ok(st) }

#[tauri::command] async fn install_version_to_instance(app: AppHandle, instance_id: String, project_id: String, version_id: String) -> Result<InstanceState, String> {
    let c = modrinth_client()?;
    app.emit("install:progress", serde_json::json!({"instance_id": instance_id, "step":"fetch-version", "progress": 0.1})).map_err(|e| e.to_string())?;
    let version = c.get(format!("https://api.modrinth.com/v2/version/{version_id}")).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json::<serde_json::Value>().await.map_err(|e|e.to_string())?;
    let file = serde_json::from_value::<VersionFile>(version["files"][0].clone()).map_err(|e| e.to_string())?;
    let bytes = c.get(&file.url).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.bytes().await.map_err(|e|e.to_string())?;
    if let Some(expected) = file.hashes.get("sha1") { if sha1_hex(&bytes) != *expected { return Err("Hash verification failed".into()); } }
    let mods_dir = instance_dir(&app, &instance_id)?.join("mods"); fs::create_dir_all(&mods_dir).map_err(|e|e.to_string())?;
    fs::write(mods_dir.join(&file.filename), &bytes).map_err(|e|e.to_string())?;
    let mut st = load_instance_state(&app, &instance_id)?;
    st.mods.retain(|m| m.file_name != file.filename);
    st.mods.push(InstanceModEntry { file_name: file.filename.clone(), project_id, version_id, enabled: true, path: format!("mods/{}", file.filename) });
    save_instance_state(&app, &instance_id, &st)?;
    app.emit("install:progress", serde_json::json!({"instance_id": instance_id, "step":"done", "progress": 1.0})).map_err(|e| e.to_string())?;
    Ok(st)
}

#[tauri::command] async fn import_mrpack(app: AppHandle, instance_id: String, mrpack_path: String) -> Result<InstanceState, String> {
    let file = fs::File::open(&mrpack_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let manifest: MrpackManifest = serde_json::from_reader(archive.by_name("modrinth.index.json").map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let c = modrinth_client()?;
    let total = manifest.files.len().max(1);
    for (idx, f) in manifest.files.iter().enumerate() {
        app.emit("install:progress", serde_json::json!({"instance_id":instance_id,"step":"download","progress":(idx as f32)/(total as f32)})).map_err(|e|e.to_string())?;
        let url = f.downloads.first().ok_or("No download url")?;
        let bytes = c.get(url).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.bytes().await.map_err(|e|e.to_string())?;
        if let Some(expected) = f.hashes.get("sha1") { if sha1_hex(&bytes) != *expected { return Err(format!("Hash mismatch for {}", f.path)); } }
        let target = instance_dir(&app, &instance_id)?.join(&f.path);
        if let Some(parent) = target.parent() { fs::create_dir_all(parent).map_err(|e|e.to_string())?; }
        fs::write(&target, &bytes).map_err(|e|e.to_string())?;
    }
    app.emit("install:progress", serde_json::json!({"instance_id":instance_id,"step":"done","progress":1.0})).map_err(|e|e.to_string())?;
    load_instance_state(&app, &instance_id)
}

#[tauri::command] fn detect_java_installations() -> Result<Vec<JavaInstallation>, String> { let mut seen = std::collections::HashSet::new(); let mut installs = vec![]; for p in candidate_java_paths() { if !p.contains("java") || (!Path::new(&p).exists() && p != "java") { continue; } if let Some(info) = probe_java(&p) { let key = format!("{}:{}", info.path, info.version); if seen.insert(key) { installs.push(info); } } } installs.sort_by_key(|j| std::cmp::Reverse(j.major)); Ok(installs) }
#[tauri::command] fn recommend_java_for_mc(mc_version: String) -> Result<JavaRecommendation, String> { let required = required_java_for_mc(&mc_version); let installed = detect_java_installations()?; let matched = installed.into_iter().find(|j| j.major == required || (required == 8 && j.major >= 8)); Ok(JavaRecommendation { mc_version, required_major: required, installed_match: matched }) }
#[tauri::command] async fn download_adoptium_java(app: AppHandle, major: u32) -> Result<String, String> { let os = if cfg!(target_os = "windows") { "windows" } else if cfg!(target_os = "macos") { "mac" } else { "linux" }; let arch = if cfg!(target_arch = "aarch64") { "aarch64" } else { "x64" }; let url = format!("https://api.adoptium.net/v3/binary/latest/{major}/ga/{os}/{arch}/jdk/hotspot/normal/eclipse"); let target = app_file(&app, &format!("java/jdk-{major}-{os}-{arch}.tar.gz"))?; if let Some(parent) = target.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; } let bytes = modrinth_client()?.get(url).send().await.map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?.bytes().await.map_err(|e| e.to_string())?; fs::write(&target, &bytes).map_err(|e| e.to_string())?; Ok(target.to_string_lossy().to_string()) }

#[tauri::command] async fn search_projects(query: String, project_type: String, categories: Option<Vec<String>>, game_version: Option<String>, loader: Option<String>, limit: Option<u32>, offset: Option<u32>, sort: Option<String>) -> Result<ModrinthSearchResponse, String> { let mut facets = vec![vec![format!("project_type:{project_type}")]]; if let Some(v) = categories { for c in v { facets.push(vec![format!("categories:{c}")]); } } if let Some(v) = game_version { facets.push(vec![format!("versions:{v}")]); } if let Some(v) = loader { facets.push(vec![format!("categories:{v}")]); } let client = modrinth_client()?; client.get("https://api.modrinth.com/v2/search").query(&[("query",query),("limit",limit.unwrap_or(20).to_string()),("offset",offset.unwrap_or(0).to_string()),("facets",serde_json::to_string(&facets).map_err(|e|e.to_string())?), ("index", sort.unwrap_or("relevance".into()))]).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json::<ModrinthSearchResponse>().await.map_err(|e|e.to_string()) }
#[tauri::command] async fn get_project(id_or_slug: String) -> Result<ModrinthProject,String>{ modrinth_client()?.get(format!("https://api.modrinth.com/v2/project/{id_or_slug}")).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string()) }
#[tauri::command] async fn get_project_versions(project_id: String) -> Result<Vec<ModrinthVersion>,String>{ modrinth_client()?.get(format!("https://api.modrinth.com/v2/project/{project_id}/version")).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string()) }
#[tauri::command] async fn get_tags() -> Result<HashMap<String, serde_json::Value>, String> { let c=modrinth_client()?; let cats:serde_json::Value=c.get("https://api.modrinth.com/v2/tag/category").send().await.map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())?; let vers:serde_json::Value=c.get("https://api.modrinth.com/v2/tag/game_version").send().await.map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())?; let loaders:serde_json::Value=c.get("https://api.modrinth.com/v2/tag/loader").send().await.map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())?; Ok(HashMap::from([(String::from("categories"),cats),(String::from("game_versions"),vers),(String::from("loaders"),loaders)])) }


#[derive(Debug, Serialize, Deserialize, Clone)]
struct RunningInstance { instance_id: String, pid: u32, started_at: String }
fn load_running(app: &AppHandle) -> Result<Vec<RunningInstance>, String> { load_json(app_file(app, "running_instances.json")?) }
fn save_running(app: &AppHandle, state: &[RunningInstance]) -> Result<(), String> { save_json(app_file(app, "running_instances.json")?, &state) }

#[tauri::command]
fn get_running_instances(app: AppHandle) -> Result<Vec<RunningInstance>, String> { load_running(&app) }

#[tauri::command]
fn stop_instance(app: AppHandle, instance_id: String) -> Result<bool, String> {
    let mut running = load_running(&app)?;
    if let Some(info) = running.iter().find(|r| r.instance_id == instance_id).cloned() {
        #[cfg(target_os = "windows")]
        let _ = Command::new("taskkill").args(["/PID", &info.pid.to_string(), "/F"]).output();
        #[cfg(not(target_os = "windows"))]
        let _ = Command::new("kill").args(["-9", &info.pid.to_string()]).output();
        running.retain(|r| r.instance_id != instance_id);
        save_running(&app, &running)?;
        return Ok(true);
    }
    Ok(false)
}

#[tauri::command]
fn launch_instance(app: AppHandle, instance_id: String) -> Result<String, String> {
    let instances = load_instances(&app)?;
    let instance = instances.instances.into_iter().find(|i| i.id == instance_id).ok_or("Instance not found")?;
    let mut state = load_instance_state(&app, &instance_id)?;
    let java = state.settings.java_path.clone().or_else(|| detect_java_installations().ok().and_then(|j| j.first().map(|x| x.path.clone()))).unwrap_or_else(|| "java".into());

    if let Some(pre) = &state.settings.pre_launch_hook { let _ = Command::new("sh").arg("-lc").arg(pre).output(); }
    let mut child = Command::new(&java)
        .arg(format!("-Xmx{}M", state.settings.memory_mb.max(1024)))
        .arg("-version")
        .spawn()
        .map_err(|e| format!("Launch failed: {e}"))?;
    let pid = child.id();
    std::thread::spawn(move || { let _ = child.wait(); });

    state.logs.push(format!("Launching {} {} with {} (offline stub)", instance.name, instance.mc_version, java));
    save_instance_state(&app, &instance_id, &state)?;
    app.emit("launch:log", serde_json::json!({"instance_id":instance_id,"line":"Process started"})).map_err(|e| e.to_string())?;

    let mut running = load_running(&app)?;
    running.retain(|r| r.instance_id != instance_id);
    running.push(RunningInstance { instance_id, pid, started_at: chrono::Utc::now().to_rfc3339() });
    save_running(&app, &running)?;
    Ok("Launched (java -version placeholder pipeline ready for full MC args)".into())
}
fn main(){ tauri::Builder::default().invoke_handler(tauri::generate_handler![get_accounts,add_offline_account,delete_account,set_active_account,list_instances,create_instance,delete_instance,get_instance_state,set_instance_settings,toggle_instance_mod,remove_instance_mod,install_version_to_instance,import_mrpack,detect_java_installations,recommend_java_for_mc,download_adoptium_java,search_projects,get_project,get_project_versions,get_tags,launch_instance,stop_instance,get_running_instances]).run(tauri::generate_context!()).expect("error while running tauri application"); }
