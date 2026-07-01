use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use chrono::{DateTime, Local, Utc};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

use super::{
    foreground::{detect_editor, sample_foreground},
    storage,
    types::{
        ActivitySegment, TrackingSettings, TrackingStatus,
    },
};

const EDITOR_FOCUS_GRACE_SECS: u64 = 90;

pub struct ActivityTracker {
    enabled: Arc<AtomicBool>,
    status: Arc<Mutex<TrackingStatus>>,
    settings: Arc<Mutex<TrackingSettings>>,
    segments: Arc<Mutex<Vec<ActivitySegment>>>,
    stop_flag: Arc<AtomicBool>,
    thread: Mutex<Option<thread::JoinHandle<()>>>,
    own_process_id: u32,
    last_editor_active_at: Arc<Mutex<Option<DateTime<Utc>>>>,
}

impl ActivityTracker {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            status: Arc::new(Mutex::new(TrackingStatus {
                enabled: false,
                last_error: None,
                open_segment_id: None,
            })),
            settings: Arc::new(Mutex::new(TrackingSettings::default())),
            segments: Arc::new(Mutex::new(Vec::new())),
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread: Mutex::new(None),
            own_process_id: std::process::id(),
            last_editor_active_at: Arc::new(Mutex::new(None)),
        }
    }

    pub fn load_segments(&self, app: &AppHandle) -> Result<Vec<ActivitySegment>, String> {
        let path = segments_path(app)?;
        let segments = read_json_file(&path, Vec::new())?;
        *self.segments.lock().map_err(|_| lock_error())? = segments.clone();
        Ok(segments)
    }

    pub fn save_segments(&self, app: &AppHandle, segments: Vec<ActivitySegment>) -> Result<(), String> {
        let path = segments_path(app)?;
        write_json_file(&path, &segments)?;
        *self.segments.lock().map_err(|_| lock_error())? = segments;
        Ok(())
    }

    pub fn get_status(&self) -> Result<TrackingStatus, String> {
        self.status
            .lock()
            .map(|status| status.clone())
            .map_err(|_| lock_error())
    }

    pub fn get_settings(&self) -> Result<TrackingSettings, String> {
        self.settings
            .lock()
            .map(|settings| settings.clone())
            .map_err(|_| lock_error())
    }

    pub fn set_settings(&self, settings: TrackingSettings) -> Result<(), String> {
        *self.settings.lock().map_err(|_| lock_error())? = settings;
        Ok(())
    }

    pub fn start(&self, app: AppHandle) -> Result<(), String> {
        if self.enabled.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.load_segments(&app)?;
        self.stop_flag.store(false, Ordering::SeqCst);
        self.enabled.store(true, Ordering::SeqCst);

        {
            let mut status = self.status.lock().map_err(|_| lock_error())?;
            status.enabled = true;
            status.last_error = None;
        }

        let enabled = Arc::clone(&self.enabled);
        let stop_flag = Arc::clone(&self.stop_flag);
        let status = Arc::clone(&self.status);
        let settings = Arc::clone(&self.settings);
        let segments = Arc::clone(&self.segments);
        let own_process_id = self.own_process_id;
        let last_editor_active_at = Arc::clone(&self.last_editor_active_at);

        let handle = thread::spawn(move || {
            poll_loop(
                app,
                enabled,
                stop_flag,
                status,
                settings,
                segments,
                own_process_id,
                last_editor_active_at,
            );
        });

        *self.thread.lock().map_err(|_| lock_error())? = Some(handle);
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        if !self.enabled.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.stop_flag.store(true, Ordering::SeqCst);
        self.enabled.store(false, Ordering::SeqCst);

        if let Some(handle) = self.thread.lock().map_err(|_| lock_error())?.take() {
            let _ = handle.join();
        }

        let mut status = self.status.lock().map_err(|_| lock_error())?;
        status.enabled = false;
        status.open_segment_id = None;
        Ok(())
    }
}

fn poll_loop(
    app: AppHandle,
    enabled: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    status: Arc<Mutex<TrackingStatus>>,
    settings: Arc<Mutex<TrackingSettings>>,
    segments: Arc<Mutex<Vec<ActivitySegment>>>,
    own_process_id: u32,
    last_editor_active_at: Arc<Mutex<Option<DateTime<Utc>>>>,
) {
    while enabled.load(Ordering::SeqCst) && !stop_flag.load(Ordering::SeqCst) {
        let poll_interval = settings
            .lock()
            .map(|value| value.poll_interval_secs)
            .unwrap_or(10)
            .max(5);

        if let Err(error) = sample_once(
            &app,
            &status,
            &settings,
            &segments,
            own_process_id,
            &last_editor_active_at,
        ) {
            if let Ok(mut status) = status.lock() {
                status.last_error = Some(error);
            }
        }

        thread::sleep(Duration::from_secs(poll_interval));
    }
}

fn sample_once(
    app: &AppHandle,
    status: &Arc<Mutex<TrackingStatus>>,
    settings: &Arc<Mutex<TrackingSettings>>,
    segments: &Arc<Mutex<Vec<ActivitySegment>>>,
    own_process_id: u32,
    last_editor_active_at: &Arc<Mutex<Option<DateTime<Utc>>>>,
) -> Result<(), String> {
    let settings = settings.lock().map_err(|_| lock_error())?.clone();
    let foreground = sample_foreground()?;
    let is_self = foreground.process_id == own_process_id;
    let editor = detect_editor(&foreground, &settings.enabled_editors);
    let now_dt = Utc::now();
    let now = now_dt.to_rfc3339();
    let date_key = Local::now().format("%Y-%m-%d").to_string();

    let mut segments = segments.lock().map_err(|_| lock_error())?;
    close_stale_open_segment(&mut segments, &now);

    if let Some(editor) = editor {
        if let Ok(mut last_active) = last_editor_active_at.lock() {
            *last_active = Some(now_dt);
        }

        let workspace = storage::read_active_workspace(editor)?;
        if let Some(workspace) = workspace {
            start_or_continue_segment(
                &mut segments,
                status,
                &now,
                &date_key,
                editor,
                workspace.key,
                workspace.label,
            )?;
        } else if let Some(segment) = segments.iter_mut().find(|segment| segment.end.is_none()) {
            segment.end = Some(now.clone());
            if let Ok(mut status) = status.lock() {
                status.open_segment_id = None;
            }
        }
    } else if is_self && segments.iter().any(|segment| segment.end.is_none()) {
        if let Ok(mut last_active) = last_editor_active_at.lock() {
            *last_active = Some(now_dt);
        }

        if let Ok(mut status) = status.lock() {
            status.last_error = None;
        }
    } else if within_editor_grace(last_editor_active_at, now_dt) {
        if let Ok(mut status) = status.lock() {
            status.last_error = None;
        }
    } else if let Some(segment) = segments.iter_mut().find(|segment| segment.end.is_none()) {
        segment.end = Some(now.clone());
        if let Ok(mut status) = status.lock() {
            status.open_segment_id = None;
        }
    }

    let path = segments_path(app)?;
    let snapshot = segments.clone();
    write_json_file(&path, &snapshot)?;
    let _ = app.emit("activity-segment-updated", snapshot);
    Ok(())
}

fn within_editor_grace(
    last_editor_active_at: &Arc<Mutex<Option<DateTime<Utc>>>>,
    now: DateTime<Utc>,
) -> bool {
    let Ok(last_active) = last_editor_active_at.lock() else {
        return false;
    };

    last_active
        .map(|timestamp| {
            now.signed_duration_since(timestamp)
                .num_seconds()
                .max(0) as u64
                <= EDITOR_FOCUS_GRACE_SECS
        })
        .unwrap_or(false)
}

fn start_or_continue_segment(
    segments: &mut Vec<ActivitySegment>,
    status: &Arc<Mutex<TrackingStatus>>,
    now: &str,
    date_key: &str,
    editor: super::types::EditorKind,
    workspace_key: String,
    workspace_label: String,
) -> Result<(), String> {
    let open = segments.iter_mut().find(|segment| segment.end.is_none());

    let should_start_new = match open {
        Some(segment) => segment.workspace_key != workspace_key || segment.editor != editor,
        None => true,
    };

    if should_start_new {
        if let Some(segment) = segments.iter_mut().find(|segment| segment.end.is_none()) {
            segment.end = Some(now.to_string());
        }

        let segment = ActivitySegment {
            id: Uuid::new_v4().to_string(),
            date: date_key.to_string(),
            editor,
            workspace_key,
            workspace_label,
            start: now.to_string(),
            end: None,
        };

        segments.push(segment.clone());

        if let Ok(mut status) = status.lock() {
            status.open_segment_id = Some(segment.id.clone());
            status.last_error = None;
        }
    } else if let Ok(mut status) = status.lock() {
        status.last_error = None;
    }

    Ok(())
}

fn close_stale_open_segment(segments: &mut [ActivitySegment], now: &str) {
    let today = Local::now().format("%Y-%m-%d").to_string();

    for segment in segments.iter_mut().filter(|segment| segment.end.is_none()) {
        if segment.date != today {
            segment.end = Some(now.to_string());
        }
    }
}

pub fn segments_path(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_file(app, "activity-segments.json")
}

pub fn mappings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_file(app, "workspace-mappings.json")
}

pub fn suggestion_state_path(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_file(app, "suggestion-state.json")
}

pub fn tracking_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_file(app, "tracking-settings.json")
}

fn app_data_file(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;

    fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;
    Ok(app_data_dir.join(filename))
}

pub fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path, default: T) -> Result<T, String> {
    if !path.exists() {
        return Ok(default);
    }

    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;

    if contents.trim().is_empty() {
        return Ok(default);
    }

    serde_json::from_str(&contents).map_err(|error| error.to_string())
}

pub fn write_json_file<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let contents = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, contents).map_err(|error| error.to_string())
}

fn lock_error() -> String {
    "Activity tracker state is unavailable.".to_string()
}
