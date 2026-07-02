use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RmProjectCatalogEntry {
    pub assignable_id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_project_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RmCatalogCache {
    pub fetched_at: String,
    #[serde(default)]
    pub categories: Vec<String>,
    pub projects: Vec<RmProjectCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RmLinkedUser {
    pub id: i64,
    pub email: String,
    pub display_name: String,
    pub linked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RmSettings {
    pub region: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_fetched_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_migration_completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_user: Option<RmLinkedUser>,
}

impl Default for RmSettings {
    fn default() -> Self {
        Self {
            region: "us".to_string(),
            catalog_fetched_at: None,
            legacy_migration_completed_at: None,
            linked_user: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RmUserSummary {
    pub id: i64,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RmUserCandidates {
    pub candidates: Vec<RmUserSummary>,
}

#[derive(Debug, Deserialize)]
pub struct RmUser {
    pub id: i64,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SyncScope {
    Changed,
    Failed,
    Selected,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDayRequest {
    pub date: String,
    pub dry_run: bool,
    pub scope: SyncScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_entry_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPreviewRow {
    pub entry_id: String,
    pub project: String,
    pub category: String,
    pub hours: f64,
    pub action: String,
    pub hash: String,
    pub included: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFailure {
    pub entry_id: String,
    pub date: String,
    pub project: String,
    pub category: String,
    pub error: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDayResult {
    pub succeeded: usize,
    pub skipped: usize,
    pub failed: Vec<SyncFailure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<Vec<SyncPreviewRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_user: Option<RmLinkedUser>,
}

#[derive(Debug, Deserialize)]
pub struct RmCreatedTimeEntry {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RmCatalogProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct RmPagedResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub paging: Option<RmPaging>,
}

#[derive(Debug, Deserialize)]
pub struct RmPaging {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub next: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RmProject {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub project_code: Option<String>,
    #[serde(default)]
    pub phase_name: Option<String>,
    #[serde(default)]
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub client: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RmBudgetCategory {
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RmBudgetItem {
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RmTimeEntry {
    #[serde(default)]
    pub task: Option<String>,
}
