use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use tauri::AppHandle;

const USER_AGENT_VALUE: &str = "mcapp-launcher/0.1.0 (hello@example.com)";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OfflineAccount {
    id: String,
    username: String,
    kind: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AccountsState { active_account_id: Option<String>, accounts: Vec<OfflineAccount> }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct InstanceSummary {
    id: String,
    name: String,
    mc_version: String,
    loader: String,
    icon: Option<String>,
    last_played: Option<String>,
    group: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct InstancesState { instances: Vec<InstanceSummary> }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModrinthSearchResponse { hits: Vec<ModrinthHit>, total_hits: Option<u64> }
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModrinthHit {
    project_id: String, title: String, description: String, icon_url: Option<String>,
    downloads: u64, project_type: String, slug: String, author: Option<String>, categories: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModrinthProject {
    id: String, slug: Option<String>, title: String, description: String, body: Option<String>,
    project_type: String, icon_url: Option<String>, downloads: Option<u64>, followers: Option<u64>, categories: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModrinthVersion { id: String, name: String, version_number: String, game_versions: Vec<String>, loaders: Vec<String>, date_published: String }

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

#[tauri::command] fn get_accounts(app: AppHandle) -> Result<AccountsState, String> { load_accounts(&app) }
#[tauri::command] fn add_offline_account(app: AppHandle, username: String) -> Result<AccountsState, String> { let mut s = load_accounts(&app)?; let id = offline_uuid(&username); if !s.accounts.iter().any(|a| a.id==id) { s.accounts.push(OfflineAccount{id:id.clone(), username, kind:"offline".into()}); } s.active_account_id=Some(id); save_accounts(&app,&s)?; Ok(s) }
#[tauri::command] fn delete_account(app: AppHandle, account_id: String) -> Result<AccountsState, String> { let mut s = load_accounts(&app)?; s.accounts.retain(|a| a.id!=account_id); if s.active_account_id.as_deref()==Some(&account_id){ s.active_account_id=s.accounts.first().map(|a|a.id.clone()); } save_accounts(&app,&s)?; Ok(s) }
#[tauri::command] fn set_active_account(app: AppHandle, account_id: String) -> Result<AccountsState, String> { let mut s = load_accounts(&app)?; if s.accounts.iter().any(|a|a.id==account_id){ s.active_account_id=Some(account_id); save_accounts(&app,&s)?;} Ok(s) }

#[tauri::command] fn list_instances(app: AppHandle) -> Result<InstancesState, String> { load_instances(&app) }
#[tauri::command] fn create_instance(app: AppHandle, name: String, mc_version: String, loader: String) -> Result<InstancesState, String> { let mut s=load_instances(&app)?; let id = format!("{}-{}", name.to_lowercase().replace(' ',"-"), s.instances.len()+1); s.instances.push(InstanceSummary{id,name,mc_version,loader,icon:None,last_played:None,group:None}); save_instances(&app,&s)?; Ok(s)}
#[tauri::command] fn delete_instance(app: AppHandle, instance_id: String) -> Result<InstancesState, String> { let mut s=load_instances(&app)?; s.instances.retain(|i|i.id!=instance_id); save_instances(&app,&s)?; Ok(s)}

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

fn main(){ tauri::Builder::default().invoke_handler(tauri::generate_handler![get_accounts,add_offline_account,delete_account,set_active_account,list_instances,create_instance,delete_instance,search_projects,get_project,get_project_versions,get_tags]).run(tauri::generate_context!()).expect("error while running tauri application"); }
