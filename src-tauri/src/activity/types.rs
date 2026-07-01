use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditorKind {
    Cursor,
    Code,
    Vscodium,
}

impl EditorKind {
    pub fn all() -> &'static [EditorKind] {
        &[EditorKind::Cursor, EditorKind::Code, EditorKind::Vscodium]
    }

    pub fn process_names(&self) -> &'static [&'static str] {
        match self {
            EditorKind::Cursor => &["Cursor", "cursor"],
            EditorKind::Code => &["Code", "code"],
            EditorKind::Vscodium => &["VSCodium", "vscodium"],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySegment {
    pub id: String,
    pub date: String,
    pub editor: EditorKind,
    pub workspace_key: String,
    pub workspace_label: String,
    pub start: String,
    pub end: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMapping {
    pub workspace_key: String,
    pub project: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownWorkspace {
    pub workspace_key: String,
    pub workspace_label: String,
    pub editor: Option<EditorKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackingStatus {
    pub enabled: bool,
    pub last_error: Option<String>,
    pub open_segment_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackingSettings {
    #[serde(default = "default_enabled_editors")]
    pub enabled_editors: Vec<EditorKind>,
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
}

fn default_enabled_editors() -> Vec<EditorKind> {
    vec![EditorKind::Cursor, EditorKind::Code, EditorKind::Vscodium]
}

fn default_poll_interval_secs() -> u64 {
    10
}

impl Default for TrackingSettings {
    fn default() -> Self {
        Self {
            enabled_editors: default_enabled_editors(),
            poll_interval_secs: default_poll_interval_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionStateEntry {
    pub suggestion_id: String,
    pub status: SuggestionStatus,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionStatus {
    Accepted,
    Dismissed,
}

#[derive(Debug, Clone)]
pub struct ResolvedWorkspace {
    pub key: String,
    pub label: String,
}
