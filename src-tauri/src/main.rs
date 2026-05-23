use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::{Path, PathBuf}, process::Command};
use tauri::AppHandle;

const USER_AGENT_VALUE: &str = "mcapp-launcher/0.1.0 (hello@example.com)";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OfflineAccount { id: String, username: String, kind: String }
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct AccountsState { active_account_id: Option<String>, accounts: Vec<OfflineAccount> }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct InstanceSummary { id: String, name: String, mc_version: String, loader: String, icon: Option<String>, last_played: Option<String>, group: Option<String> }
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct InstancesState { instances: Vec<InstanceSummary> }

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

fn app_file(app: &AppHandle, name: &str) -> Result<PathBuf, String> { let dir = app.path().app_data_dir().map_err(|e| e.to_string())?; fs::create_dir_all(&dir).map_err(|e| e.to_string())?; Ok(dir.join(name)) }
fn load_json<T: for<'de> Deserialize<'de> + Default>(path: PathBuf) -> Result<T, String> { if !path.exists() { return Ok(T::default()); } serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string()) }
fn save_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> { fs::write(path, serde_json::to_string_pretty(value).map_err(|e| e.to_string())?).map_err(|e| e.to_string()) }

fn load_accounts(app: &AppHandle) -> Result<AccountsState, String> { load_json(app_file(app, "accounts.json")?) }
fn save_accounts(app: &AppHandle, state: &AccountsState) -> Result<(), String> { save_json(app_file(app, "accounts.json")?, state) }
fn load_instances(app: &AppHandle) -> Result<InstancesState, String> { load_json(app_file(app, "instances.json")?) }
fn save_instances(app: &AppHandle, state: &InstancesState) -> Result<(), String> { save_json(app_file(app, "instances.json")?, state) }

fn offline_uuid(name: &str) -> String {
    let digest = md5::compute(format!("OfflinePlayer:{name}").as_bytes());
    let mut b = digest.0; b[6] = (b[6] & 0x0f) | 0x30; b[8] = (b[8] & 0x3f) | 0x80;
    format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", u32::from_be_bytes([b[0],b[1],b[2],b[3]]), u16::from_be_bytes([b[4],b[5]]), u16::from_be_bytes([b[6],b[7]]), u16::from_be_bytes([b[8],b[9]]), u64::from_be_bytes([0,0,b[10],b[11],b[12],b[13],b[14],b[15]]))
}

fn modrinth_client() -> Result<reqwest::Client, String> { let mut h = HeaderMap::new(); h.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE)); reqwest::Client::builder().default_headers(h).build().map_err(|e| e.to_string()) }

fn parse_major(version: &str) -> Option<u32> {
    if let Some(stripped) = version.split('"').nth(1) {
        let token = stripped.split('.').next()?;
        return token.parse::<u32>().ok();
    }
    None
}

fn candidate_java_paths() -> Vec<String> {
    let mut out = vec!["java".to_string()];
    if let Ok(home) = std::env::var("JAVA_HOME") { out.push(format!("{home}/bin/java")); }
    if cfg!(target_os = "windows") {
        out.extend(["C:/Program Files/Java/jdk-21/bin/java.exe".into(), "C:/Program Files/Eclipse Adoptium/jdk-21/bin/java.exe".into()]);
    } else if cfg!(target_os = "macos") {
        out.extend(["/usr/bin/java".into(), "/Library/Java/JavaVirtualMachines/temurin-21.jdk/Contents/Home/bin/java".into()]);
    } else {
        out.extend(["/usr/bin/java".into(), "/usr/lib/jvm/java-21-openjdk-amd64/bin/java".into(), "/usr/lib/jvm/temurin-21-jdk-amd64/bin/java".into()]);
    }
    out
}

fn probe_java(path: &str) -> Option<JavaInstallation> {
    let output = Command::new(path).arg("-version").output().ok()?;
    let text = String::from_utf8_lossy(&output.stderr).to_string() + &String::from_utf8_lossy(&output.stdout);
    let first = text.lines().next()?.to_string();
    let major = parse_major(&first)?;
    Some(JavaInstallation { path: path.to_string(), version: first, major })
}

fn required_java_for_mc(mc: &str) -> u32 {
    let minor = mc.split('.').nth(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(21);
    if minor >= 20 { 21 } else if minor >= 18 { 17 } else { 8 }
}

#[tauri::command] fn get_accounts(app: AppHandle) -> Result<AccountsState, String> { load_accounts(&app) }
#[tauri::command] fn add_offline_account(app: AppHandle, username: String) -> Result<AccountsState, String> { let mut s = load_accounts(&app)?; let id = offline_uuid(&username); if !s.accounts.iter().any(|a| a.id==id) { s.accounts.push(OfflineAccount{id:id.clone(), username, kind:"offline".into()}); } s.active_account_id=Some(id); save_accounts(&app,&s)?; Ok(s) }
#[tauri::command] fn delete_account(app: AppHandle, account_id: String) -> Result<AccountsState, String> { let mut s = load_accounts(&app)?; s.accounts.retain(|a| a.id!=account_id); if s.active_account_id.as_deref()==Some(&account_id){ s.active_account_id=s.accounts.first().map(|a|a.id.clone()); } save_accounts(&app,&s)?; Ok(s) }
#[tauri::command] fn set_active_account(app: AppHandle, account_id: String) -> Result<AccountsState, String> { let mut s = load_accounts(&app)?; if s.accounts.iter().any(|a|a.id==account_id){ s.active_account_id=Some(account_id); save_accounts(&app,&s)?;} Ok(s) }

#[tauri::command] fn list_instances(app: AppHandle) -> Result<InstancesState, String> { load_instances(&app) }
#[tauri::command] fn create_instance(app: AppHandle, name: String, mc_version: String, loader: String) -> Result<InstancesState, String> { let mut s=load_instances(&app)?; let id = format!("{}-{}", name.to_lowercase().replace(' ',"-"), s.instances.len()+1); s.instances.push(InstanceSummary{id,name,mc_version,loader,icon:None,last_played:None,group:None}); save_instances(&app,&s)?; Ok(s)}
#[tauri::command] fn delete_instance(app: AppHandle, instance_id: String) -> Result<InstancesState, String> { let mut s=load_instances(&app)?; s.instances.retain(|i|i.id!=instance_id); save_instances(&app,&s)?; Ok(s)}

#[tauri::command] fn detect_java_installations() -> Result<Vec<JavaInstallation>, String> {
    let mut seen = std::collections::HashSet::new();
    let mut installs = vec![];
    for p in candidate_java_paths() {
        if !p.contains("java") || (!Path::new(&p).exists() && p != "java") { continue; }
        if let Some(info) = probe_java(&p) {
            let key = format!("{}:{}", info.path, info.version);
            if seen.insert(key) { installs.push(info); }
        }
    }
    installs.sort_by_key(|j| std::cmp::Reverse(j.major));
    Ok(installs)
}

#[tauri::command] fn recommend_java_for_mc(mc_version: String) -> Result<JavaRecommendation, String> {
    let required = required_java_for_mc(&mc_version);
    let installed = detect_java_installations()?;
    let matched = installed.into_iter().find(|j| j.major == required || (required == 8 && j.major >= 8));
    Ok(JavaRecommendation { mc_version, required_major: required, installed_match: matched })
}

#[tauri::command] async fn download_adoptium_java(app: AppHandle, major: u32) -> Result<String, String> {
    let os = if cfg!(target_os = "windows") { "windows" } else if cfg!(target_os = "macos") { "mac" } else { "linux" };
    let arch = if cfg!(target_arch = "aarch64") { "aarch64" } else { "x64" };
    let url = format!("https://api.adoptium.net/v3/binary/latest/{major}/ga/{os}/{arch}/jdk/hotspot/normal/eclipse");
    let target = app_file(&app, &format!("java/jdk-{major}-{os}-{arch}.tar.gz"))?;
    if let Some(parent) = target.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let bytes = modrinth_client()?.get(url).send().await.map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?.bytes().await.map_err(|e| e.to_string())?;
    fs::write(&target, &bytes).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
async fn search_projects(query: String, project_type: String, categories: Option<Vec<String>>, game_version: Option<String>, loader: Option<String>, limit: Option<u32>, offset: Option<u32>, sort: Option<String>) -> Result<ModrinthSearchResponse, String> {
    let mut facets = vec![vec![format!("project_type:{project_type}")]];
    if let Some(v) = categories { for c in v { facets.push(vec![format!("categories:{c}")]); } }
    if let Some(v) = game_version { facets.push(vec![format!("versions:{v}")]); }
    if let Some(v) = loader { facets.push(vec![format!("categories:{v}")]); }
    let client = modrinth_client()?;
    client.get("https://api.modrinth.com/v2/search").query(&[("query",query),("limit",limit.unwrap_or(20).to_string()),("offset",offset.unwrap_or(0).to_string()),("facets",serde_json::to_string(&facets).map_err(|e|e.to_string())?), ("index", sort.unwrap_or("relevance".into()))]).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json::<ModrinthSearchResponse>().await.map_err(|e|e.to_string())
}
#[tauri::command] async fn get_project(id_or_slug: String) -> Result<ModrinthProject,String>{ modrinth_client()?.get(format!("https://api.modrinth.com/v2/project/{id_or_slug}")).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string()) }
#[tauri::command] async fn get_project_versions(project_id: String) -> Result<Vec<ModrinthVersion>,String>{ modrinth_client()?.get(format!("https://api.modrinth.com/v2/project/{project_id}/version")).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string()) }
#[tauri::command] async fn get_tags() -> Result<HashMap<String, serde_json::Value>, String> { let c=modrinth_client()?; let cats:serde_json::Value=c.get("https://api.modrinth.com/v2/tag/category").send().await.map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())?; let vers:serde_json::Value=c.get("https://api.modrinth.com/v2/tag/game_version").send().await.map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())?; let loaders:serde_json::Value=c.get("https://api.modrinth.com/v2/tag/loader").send().await.map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())?; Ok(HashMap::from([(String::from("categories"),cats),(String::from("game_versions"),vers),(String::from("loaders"),loaders)])) }

fn main(){ tauri::Builder::default().invoke_handler(tauri::generate_handler![get_accounts,add_offline_account,delete_account,set_active_account,list_instances,create_instance,delete_instance,detect_java_installations,recommend_java_for_mc,download_adoptium_java,search_projects,get_project,get_project_versions,get_tags]).run(tauri::generate_context!()).expect("error while running tauri application"); }
