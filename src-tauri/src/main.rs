use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{fs, path::PathBuf};
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OfflineAccount {
    id: String,
    username: String,
    kind: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AccountsState {
    active_account_id: Option<String>,
    accounts: Vec<OfflineAccount>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthHit>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModrinthHit {
    project_id: String,
    title: String,
    description: String,
    icon_url: Option<String>,
    downloads: u64,
    project_type: String,
    slug: String,
}

fn accounts_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("accounts.json"))
}

fn load_accounts(app: &tauri::AppHandle) -> Result<AccountsState, String> {
    let path = accounts_file(app)?;
    if !path.exists() {
        return Ok(AccountsState { active_account_id: None, accounts: vec![] });
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn save_accounts(app: &tauri::AppHandle, state: &AccountsState) -> Result<(), String> {
    let path = accounts_file(app)?;
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn offline_uuid(name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(format!("OfflinePlayer:{name}").as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]])
    )
}

#[tauri::command]
fn get_accounts(app: tauri::AppHandle) -> Result<AccountsState, String> { load_accounts(&app) }

#[tauri::command]
fn add_offline_account(app: tauri::AppHandle, username: String) -> Result<AccountsState, String> {
    let mut state = load_accounts(&app)?;
    let id = offline_uuid(&username);
    if !state.accounts.iter().any(|a| a.id == id) {
        state.accounts.push(OfflineAccount { id: id.clone(), username, kind: "offline".into() });
    }
    state.active_account_id = Some(id);
    save_accounts(&app, &state)?;
    Ok(state)
}

#[tauri::command]
fn delete_account(app: tauri::AppHandle, account_id: String) -> Result<AccountsState, String> {
    let mut state = load_accounts(&app)?;
    state.accounts.retain(|a| a.id != account_id);
    if state.active_account_id.as_deref() == Some(&account_id) {
        state.active_account_id = state.accounts.first().map(|a| a.id.clone());
    }
    save_accounts(&app, &state)?;
    Ok(state)
}

#[tauri::command]
fn set_active_account(app: tauri::AppHandle, account_id: String) -> Result<AccountsState, String> {
    let mut state = load_accounts(&app)?;
    if state.accounts.iter().any(|a| a.id == account_id) {
        state.active_account_id = Some(account_id);
        save_accounts(&app, &state)?;
    }
    Ok(state)
}

#[tauri::command]
async fn search_projects(query: String, project_type: String, page: u32) -> Result<Vec<ModrinthHit>, String> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("mcapp-launcher/0.1.0 (hello@example.com)"));
    let client = reqwest::Client::builder().default_headers(headers).build().map_err(|e| e.to_string())?;

    let index = page * 20;
    let facets = format!("[[\"project_type:{project_type}\"]]");
    let response = client
        .get("https://api.modrinth.com/v2/search")
        .query(&[("query", query), ("limit", "20".to_string()), ("index", index.to_string()), ("facets", facets)])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json::<ModrinthSearchResponse>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(response.hits)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            add_offline_account,
            delete_account,
            set_active_account,
            search_projects
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
