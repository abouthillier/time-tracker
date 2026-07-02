mod catalog;
mod client;
mod settings;
mod token;
mod types;

pub use types::{RmCatalogCache, RmSettings};

use tauri::AppHandle;

#[tauri::command]
pub fn rm_save_token(token: String) -> Result<(), String> {
    token::save_token(&token)
}

#[tauri::command]
pub fn rm_clear_token() -> Result<(), String> {
    token::clear_token()
}

#[tauri::command]
pub fn rm_has_token() -> Result<bool, String> {
    token::has_token()
}

#[tauri::command]
pub fn rm_test_connection(token: Option<String>) -> Result<(), String> {
    let token = resolve_token(token)?;
    catalog::test_connection(&token)
}

#[tauri::command]
pub fn rm_load_settings(app: AppHandle) -> Result<RmSettings, String> {
    settings::load_settings(&app)
}

#[tauri::command]
pub fn rm_load_catalog(app: AppHandle) -> Result<Option<RmCatalogCache>, String> {
    settings::load_catalog(&app)
}

#[tauri::command]
pub fn rm_refresh_catalog(app: AppHandle, token: Option<String>) -> Result<RmCatalogCache, String> {
    let token = resolve_token(token)?;
    catalog::refresh_catalog(&app, &token)
}

fn resolve_token(token: Option<String>) -> Result<String, String> {
    if let Some(value) = token {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    require_token()
}

fn require_token() -> Result<String, String> {
    token::get_token()?.ok_or_else(|| {
        "No Resource Management API token saved. Add one in Resource Management settings.".to_string()
    })
}
