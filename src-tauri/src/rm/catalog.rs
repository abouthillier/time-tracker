use std::collections::HashSet;
use std::thread;

use chrono::Utc;
use tauri::{AppHandle, Emitter};

use super::client::RmClient;
use super::settings::{load_settings, save_catalog, save_settings};
use super::types::{
    RmBudgetCategory, RmCatalogCache, RmCatalogProgress, RmPagedResponse, RmProject,
    RmProjectCatalogEntry,
};

pub fn refresh_catalog(app: &AppHandle, token: &str) -> Result<RmCatalogCache, String> {
    let client = RmClient::new(token)?;
    let projects = fetch_all_projects(&client)?;
    let total = projects.len();

    let mut catalog_projects = Vec::with_capacity(total);

    for (index, project) in projects.iter().enumerate() {
        emit_progress(
            app,
            index + 1,
            total,
            format!("Fetching categories for {}", project.name),
        );

        let categories = fetch_project_categories(&client, project.id)?;

        catalog_projects.push(RmProjectCatalogEntry {
            assignable_id: project.id,
            name: project.name.clone(),
            project_code: non_empty(project.project_code.clone()),
            phase_name: non_empty(project.phase_name.clone()),
            parent_project_id: project.parent_id,
            client: project_client_label(&project.client),
            categories,
        });

        if index + 1 < total {
            thread::sleep(RmClient::category_fetch_delay());
        }
    }

    let fetched_at = Utc::now().to_rfc3339();
    let cache = RmCatalogCache {
        fetched_at: fetched_at.clone(),
        projects: catalog_projects,
    };

    save_catalog(app, &cache)?;

    let mut settings = load_settings(app)?;
    settings.catalog_fetched_at = Some(fetched_at);
    save_settings(app, &settings)?;

    emit_progress(
        app,
        total,
        total,
        format!("Cached {} projects from Resource Management", cache.projects.len()),
    );

    Ok(cache)
}

fn fetch_all_projects(client: &RmClient) -> Result<Vec<RmProject>, String> {
    let mut projects = Vec::new();
    let mut page = 1_i64;

    loop {
        let page_str = page.to_string();
        let query = [("per_page", "1000"), ("page", page_str.as_str())];
        let response: RmPagedResponse<RmProject> = client.get_json("projects", &query)?;

        if response.data.is_empty() {
            break;
        }

        projects.extend(response.data);

        let has_next = response
            .paging
            .as_ref()
            .and_then(|paging| paging.next.as_ref())
            .is_some_and(|next| !next.is_empty());

        if !has_next {
            break;
        }

        page += 1;
    }

    projects.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(projects)
}

fn fetch_project_categories(client: &RmClient, project_id: i64) -> Result<Vec<String>, String> {
    let path = format!("projects/{project_id}/time_entry_categories");
    let query = [("per_page", "1000"), ("page", "1")];

    let response: RmPagedResponse<RmBudgetCategory> = match client.get_json(&path, &query) {
        Ok(value) => value,
        Err(_) => {
            return Ok(Vec::new());
        }
    };

    let mut categories = Vec::new();
    let mut seen = HashSet::new();

    for item in response.data {
        if let Some(category) = item.category {
            let trimmed = category.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            let key = trimmed.to_lowercase();
            if seen.insert(key) {
                categories.push(trimmed);
            }
        }
    }

    categories.sort_by(|left, right| left.cmp(right));
    Ok(categories)
}

fn emit_progress(app: &AppHandle, current: usize, total: usize, message: String) {
    let _ = app.emit(
        "rm-catalog-progress",
        RmCatalogProgress {
            current,
            total,
            message,
        },
    );
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn project_client_label(client: &Option<serde_json::Value>) -> Option<String> {
    match client {
        Some(serde_json::Value::String(text)) => non_empty(Some(text.clone())),
        Some(serde_json::Value::Object(map)) => map
            .get("name")
            .and_then(|value| value.as_str())
            .map(|text| text.to_string())
            .and_then(|text| non_empty(Some(text))),
        _ => None,
    }
}

pub fn test_connection(token: &str) -> Result<(), String> {
    let client = RmClient::new(token)?;
    let _: RmPagedResponse<serde_json::Value> =
        client.get_json("users", &[("per_page", "1"), ("page", "1")])?;
    Ok(())
}
