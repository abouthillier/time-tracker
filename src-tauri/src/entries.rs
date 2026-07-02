use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSlot {
    pub start_time: String,
    pub end_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RmEntrySync {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synced_hash: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeEntry {
    pub id: String,
    pub date: String,
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignable_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub entries: Vec<TimeSlot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rm_sync: Option<RmEntrySync>,
}

pub fn load_entries(app: &AppHandle) -> Result<Vec<TimeEntry>, String> {
    let path = entries_path(app)?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;

    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&contents).map_err(|error| error.to_string())
}

pub fn save_entries(app: &AppHandle, entries: &[TimeEntry]) -> Result<(), String> {
    let path = entries_path(app)?;
    let contents = serde_json::to_string_pretty(entries).map_err(|error| error.to_string())?;

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
