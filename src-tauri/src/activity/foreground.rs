use super::types::EditorKind;

#[derive(Debug, Clone)]
pub struct ForegroundSample {
    pub process_name: String,
    pub process_path: Option<String>,
    pub process_id: u32,
}

pub fn sample_foreground() -> Result<ForegroundSample, String> {
    let window = active_win_pos_rs::get_active_window().map_err(|()| {
        "Could not read the foreground window. Check OS permissions.".to_string()
    })?;

    Ok(ForegroundSample {
        process_name: window.app_name,
        process_path: Some(
            window
                .process_path
                .to_string_lossy()
                .into_owned(),
        ),
        process_id: window.process_id as u32,
    })
}

pub fn detect_editor(sample: &ForegroundSample, enabled: &[EditorKind]) -> Option<EditorKind> {
    let haystack = format!(
        "{} {}",
        sample.process_name.to_lowercase(),
        sample
            .process_path
            .as_deref()
            .unwrap_or_default()
            .to_lowercase()
    );

    for editor in enabled {
        for name in editor.process_names() {
            let needle = name.to_lowercase();
            if haystack.contains(&needle) {
                return Some(*editor);
            }
        }
    }

    None
}
