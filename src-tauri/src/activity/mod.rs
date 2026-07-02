mod editor;
mod foreground;
mod storage;
mod tracker;
mod types;
mod workspace;

pub use tracker::{read_json_file, write_json_file, ActivityTracker};
pub use types::{
    ActivitySegment, EditorKind, KnownWorkspace, SuggestionStateEntry,
    TrackingSettings, TrackingStatus, WorkspaceMapping,
};

use std::collections::HashSet;

use tauri::{AppHandle, State};

pub fn list_known_workspaces(app: &AppHandle) -> Result<Vec<KnownWorkspace>, String> {
    let mut seen = HashSet::new();
    let mut workspaces = Vec::new();

    let segments_path = tracker::segments_path(app)?;
    let segments: Vec<ActivitySegment> =
        tracker::read_json_file(&segments_path, Vec::new())?;

    for segment in segments {
        if seen.insert(segment.workspace_key.clone()) {
            workspaces.push(KnownWorkspace {
                workspace_key: segment.workspace_key,
                workspace_label: segment.workspace_label,
                editor: Some(segment.editor),
            });
        }
    }

    for editor in EditorKind::all() {
        let scanned = storage::scan_workspace_storage(*editor)?;
        for item in scanned {
            if seen.insert(item.key.clone()) {
                workspaces.push(KnownWorkspace {
                    workspace_key: item.key,
                    workspace_label: item.label,
                    editor: Some(*editor),
                });
            }
        }
    }

    workspaces.sort_by(|left, right| left.workspace_label.cmp(&right.workspace_label));
    Ok(workspaces)
}

#[tauri::command]
pub fn load_activity_segments(
    app: AppHandle,
    tracker: State<'_, ActivityTracker>,
) -> Result<Vec<ActivitySegment>, String> {
    tracker.load_segments(&app)
}

#[tauri::command]
pub fn load_workspace_mappings(app: AppHandle) -> Result<Vec<WorkspaceMapping>, String> {
    let path = tracker::mappings_path(&app)?;
    tracker::read_json_file(&path, Vec::new())
}

#[tauri::command]
pub fn save_workspace_mappings(
    app: AppHandle,
    mappings: Vec<WorkspaceMapping>,
) -> Result<(), String> {
    let path = tracker::mappings_path(&app)?;
    tracker::write_json_file(&path, &mappings)
}

#[tauri::command]
pub fn load_suggestion_state(app: AppHandle) -> Result<Vec<SuggestionStateEntry>, String> {
    let path = tracker::suggestion_state_path(&app)?;
    tracker::read_json_file(&path, Vec::new())
}

#[tauri::command]
pub fn save_suggestion_state(
    app: AppHandle,
    state: Vec<SuggestionStateEntry>,
) -> Result<(), String> {
    let path = tracker::suggestion_state_path(&app)?;
    tracker::write_json_file(&path, &state)
}

#[tauri::command]
pub fn load_tracking_settings(app: AppHandle) -> Result<TrackingSettings, String> {
    let path = tracker::tracking_settings_path(&app)?;
    tracker::read_json_file(&path, TrackingSettings::default())
}

#[tauri::command]
pub fn save_tracking_settings(
    app: AppHandle,
    tracker: State<'_, ActivityTracker>,
    settings: TrackingSettings,
) -> Result<(), String> {
    let path = tracker::tracking_settings_path(&app)?;
    tracker::write_json_file(&path, &settings)?;
    tracker.set_settings(settings)
}

#[tauri::command]
pub fn get_tracking_status(tracker: State<'_, ActivityTracker>) -> Result<TrackingStatus, String> {
    tracker.get_status()
}

#[tauri::command]
pub fn start_activity_tracking(
    app: AppHandle,
    tracker: State<'_, ActivityTracker>,
) -> Result<(), String> {
    tracker.start(app)
}

#[tauri::command]
pub fn stop_activity_tracking(tracker: State<'_, ActivityTracker>) -> Result<(), String> {
    tracker.stop()
}

#[tauri::command]
pub fn list_known_workspaces_command(app: AppHandle) -> Result<Vec<KnownWorkspace>, String> {
    list_known_workspaces(&app)
}
