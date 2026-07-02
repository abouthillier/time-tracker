use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::activity::{read_json_file, write_json_file};
use super::types::{RmCatalogCache, RmSettings};

const SETTINGS_FILE: &str = "rm-settings.json";
const CATALOG_FILE: &str = "rm-catalog-cache.json";

pub fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_file(app, SETTINGS_FILE)
}

pub fn catalog_path(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_file(app, CATALOG_FILE)
}

pub fn load_settings(app: &AppHandle) -> Result<RmSettings, String> {
    let path = settings_path(app)?;
    read_json_file(&path, RmSettings::default())
}

pub fn save_settings(app: &AppHandle, settings: &RmSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    write_json_file(&path, settings)
}

pub fn load_catalog(app: &AppHandle) -> Result<Option<RmCatalogCache>, String> {
    let path = catalog_path(app)?;

    if !path.exists() {
        return Ok(None);
    }

    read_json_file(
        &path,
        RmCatalogCache {
            fetched_at: String::new(),
            projects: Vec::new(),
        },
    )
    .map(Some)
}

pub fn save_catalog(app: &AppHandle, catalog: &RmCatalogCache) -> Result<(), String> {
    let path = catalog_path(app)?;
    write_json_file(&path, catalog)
}

fn app_data_file(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;

    std::fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;
    Ok(app_data_dir.join(filename))
}
