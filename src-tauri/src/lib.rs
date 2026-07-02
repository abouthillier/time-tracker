use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

mod activity;
mod rm;

use activity::{
    get_tracking_status, list_known_workspaces_command, load_activity_segments,
    load_suggestion_state, load_tracking_settings, load_workspace_mappings,
    save_suggestion_state, save_tracking_settings, save_workspace_mappings,
    start_activity_tracking, stop_activity_tracking, ActivityTracker,
};

use rm::{
    rm_clear_token, rm_has_token, rm_load_catalog, rm_load_settings, rm_refresh_catalog,
    rm_save_token, rm_test_connection,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimeSlot {
    start_time: String,
    end_time: String,
    notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimeEntry {
    id: String,
    date: String,
    project: String,
    entries: Vec<TimeSlot>,
}

#[tauri::command]
fn load_entries(app: AppHandle) -> Result<Vec<TimeEntry>, String> {
    let path = entries_path(&app)?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;

    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&contents).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_entries(app: AppHandle, entries: Vec<TimeEntry>) -> Result<(), String> {
    let path = entries_path(&app)?;
    let contents = serde_json::to_string_pretty(&entries).map_err(|error| error.to_string())?;

    fs::write(path, contents).map_err(|error| error.to_string())
}

fn entries_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;

    fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;

    Ok(app_data_dir.join("entries.json"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ActivityTracker::new())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let tracker = app.state::<ActivityTracker>();
            let settings = load_tracking_settings(app.handle().clone())?;
            tracker.set_settings(settings)?;

            let app_handle = app.handle().clone();
            tracker.start(app_handle)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_entries,
            save_entries,
            load_activity_segments,
            load_workspace_mappings,
            save_workspace_mappings,
            load_suggestion_state,
            save_suggestion_state,
            load_tracking_settings,
            save_tracking_settings,
            get_tracking_status,
            start_activity_tracking,
            stop_activity_tracking,
            list_known_workspaces_command,
            rm_save_token,
            rm_clear_token,
            rm_has_token,
            rm_test_connection,
            rm_load_settings,
            rm_load_catalog,
            rm_refresh_catalog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
