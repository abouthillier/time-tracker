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
    pub projects: Vec<RmProjectCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RmSettings {
    pub region: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_fetched_at: Option<String>,
}

impl Default for RmSettings {
    fn default() -> Self {
        Self {
            region: "us".to_string(),
            catalog_fetched_at: None,
        }
    }
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
