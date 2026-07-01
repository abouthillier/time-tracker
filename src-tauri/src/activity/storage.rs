use std::{fs, path::Path};

use serde::Deserialize;
use serde_json::Value;

use super::types::EditorKind;
use super::{editor, types::ResolvedWorkspace, workspace};

pub fn read_active_workspace(editor: EditorKind) -> Result<Option<ResolvedWorkspace>, String> {
    let path = editor::storage_json_path(editor).ok_or_else(|| {
        format!("Could not resolve storage path for editor {:?}", editor)
    })?;

    if !path.exists() {
        return Ok(None);
    }

    let contents = read_json_with_retry(&path)?;
    let value: Value = serde_json::from_str(&contents).map_err(|error| error.to_string())?;

    let windows_state = value
        .get("windowsState")
        .ok_or_else(|| "storage.json is missing windowsState".to_string())?;

    let last_active = windows_state
        .get("lastActiveWindow")
        .ok_or_else(|| "windowsState is missing lastActiveWindow".to_string())?;

    let uri = extract_workspace_uri(last_active)?;
    let Some(uri) = uri else {
        return Ok(None);
    };

    workspace::resolve_workspace_uri(&uri, editor)
}

fn read_json_with_retry(path: &Path) -> Result<String, String> {
    const MAX_ATTEMPTS: usize = 3;

    for attempt in 0..MAX_ATTEMPTS {
        match fs::read_to_string(path) {
            Ok(contents) => return Ok(contents),
            Err(_error) if attempt + 1 < MAX_ATTEMPTS => {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            Err(error) => return Err(error.to_string()),
        }
    }

    Err("Failed to read storage.json".to_string())
}

fn extract_workspace_uri(last_active: &Value) -> Result<Option<String>, String> {
    if let Some(folder) = last_active.get("folder").and_then(Value::as_str) {
        return Ok(Some(folder.to_string()));
    }

    if let Some(folder_uri) = last_active.get("folderUri") {
        if let Some(uri) = folder_uri.as_str() {
            return Ok(Some(uri.to_string()));
        }

        if let Some(external) = folder_uri.get("external").and_then(Value::as_str) {
            return Ok(Some(external.to_string()));
        }
    }

    if let Some(workspace_identifier) = last_active.get("workspaceIdentifier") {
        if let Some(config_uri) = workspace_identifier
            .get("configURIPath")
            .and_then(Value::as_str)
        {
            return Ok(Some(config_uri.to_string()));
        }
    }

    Ok(None)
}

#[derive(Debug, Deserialize)]
struct WorkspaceJsonFolder {
    path: String,
}

#[derive(Debug, Deserialize)]
struct WorkspaceJsonFile {
    folder: Option<String>,
    workspace: Option<String>,
    folders: Option<Vec<WorkspaceJsonFolder>>,
}

pub fn scan_workspace_storage(editor: EditorKind) -> Result<Vec<ResolvedWorkspace>, String> {
    let Some(dir) = editor::workspace_storage_dir(editor) else {
        return Ok(Vec::new());
    };

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut results: Vec<ResolvedWorkspace> = Vec::new();

    for entry in fs::read_dir(&dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let workspace_json = entry.path().join("workspace.json");

        if !workspace_json.exists() {
            continue;
        }

        let contents = fs::read_to_string(&workspace_json).map_err(|error| error.to_string())?;
        let parsed: WorkspaceJsonFile =
            serde_json::from_str(&contents).map_err(|error| error.to_string())?;

        let uri = parsed
            .folder
            .or(parsed.workspace)
            .or_else(|| {
                parsed.folders.as_ref().and_then(|folders| {
                    folders
                        .first()
                        .map(|folder| format!("file:///{}", folder.path.replace('\\', "/")))
                })
            });

        let Some(uri) = uri else {
            continue;
        };

        if let Ok(Some(resolved)) = workspace::resolve_workspace_uri(&uri, editor) {
            if !results.iter().any(|item| item.key == resolved.key) {
                results.push(resolved);
            }
        }
    }

    Ok(results)
}
