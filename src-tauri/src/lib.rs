use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimeEntry {
    id: String,
    date: String,
    project: String,
    start_time: String,
    end_time: String,
    notes: Option<String>,
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
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_entries, save_entries])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
