use tauri::{AppHandle, Manager};

mod activity;
mod entries;
mod rm;

use activity::{
    get_tracking_status, list_known_workspaces_command, load_activity_segments,
    load_suggestion_state, load_tracking_settings, load_workspace_mappings,
    save_suggestion_state, save_tracking_settings, save_workspace_mappings,
    start_activity_tracking, stop_activity_tracking, ActivityTracker,
};

use entries::{load_entries as load_entries_from_disk, save_entries as save_entries_to_disk, TimeEntry};

use rm::{
    rm_clear_token, rm_find_user_candidates, rm_get_linked_user, rm_has_token, rm_link_user,
    rm_load_catalog, rm_load_settings, rm_refresh_catalog, rm_save_settings, rm_save_token,
    rm_sync_day, rm_test_connection, rm_unlink_user,
};

#[tauri::command]
fn load_entries(app: AppHandle) -> Result<Vec<TimeEntry>, String> {
    load_entries_from_disk(&app)
}

#[tauri::command]
fn save_entries(app: AppHandle, entries: Vec<TimeEntry>) -> Result<(), String> {
    save_entries_to_disk(&app, &entries)
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
            rm_save_settings,
            rm_load_catalog,
            rm_refresh_catalog,
            rm_find_user_candidates,
            rm_link_user,
            rm_unlink_user,
            rm_get_linked_user,
            rm_sync_day,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
