mod catalog;
mod client;
mod identity;
mod settings;
mod sync;
mod token;
mod types;

pub use types::{RmCatalogCache, RmLinkedUser, RmSettings, RmUserCandidates, SyncDayRequest, SyncDayResult};

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
pub fn rm_save_settings(app: AppHandle, settings: RmSettings) -> Result<(), String> {
    settings::save_settings(&app, &settings)
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

#[tauri::command]
pub fn rm_find_user_candidates(email: String, token: Option<String>) -> Result<RmUserCandidates, String> {
    let token = resolve_token(token)?;
    identity::find_user_candidates(&token, &email)
}

#[tauri::command]
pub fn rm_link_user(
    app: AppHandle,
    user_id: i64,
    email: String,
    display_name: String,
) -> Result<RmLinkedUser, String> {
    identity::link_user(&app, user_id, email, display_name)
}

#[tauri::command]
pub fn rm_unlink_user(app: AppHandle) -> Result<(), String> {
    identity::unlink_user(&app)
}

#[tauri::command]
pub fn rm_get_linked_user(app: AppHandle) -> Result<Option<RmLinkedUser>, String> {
    identity::get_linked_user(&app)
}

#[tauri::command]
pub fn rm_sync_day(app: AppHandle, request: SyncDayRequest) -> Result<SyncDayResult, String> {
    sync::sync_day(&app, request)
}

fn resolve_token(token: Option<String>) -> Result<String, String> {
    if let Some(value) = token {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    identity::require_token()
}
