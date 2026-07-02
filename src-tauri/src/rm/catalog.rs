use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use chrono::Utc;
use log::warn;
use tauri::{AppHandle, Emitter};

use super::client::RmClient;
use super::settings::{load_settings, save_catalog, save_settings};
use super::types::{
    RmBudgetCategory, RmBudgetItem, RmCatalogCache, RmCatalogProgress, RmPagedResponse, RmProject,
    RmProjectCatalogEntry, RmTimeEntry,
};

const PRIORITY_PROJECT_HINTS: &[&str] = &[
    "ELC",
    "NCC Separation",
    "National Geographic Explorer",
    "Trivium Internal",
    "Museum of American Finance",
    "US Capitol",
];

const MIN_USEFUL_CATEGORIES: usize = 8;
const MAX_CATEGORY_PROBE_PROJECTS: usize = 25;
const CATEGORY_PROBE_DELAY: Duration = Duration::from_millis(100);

pub fn refresh_catalog(app: &AppHandle, token: &str) -> Result<RmCatalogCache, String> {
    let client = RmClient::new(token)?;

    emit_progress(app, 0, 2, "Fetching projects from Resource Management".to_string());
    let projects = fetch_all_projects(&client)?;

    emit_progress(
        app,
        1,
        2,
        "Discovering timesheet categories from Resource Management".to_string(),
    );
    let categories = discover_timesheet_categories(&client, &projects, app)?;

    if categories.len() < 3 {
        warn!(
            "Only {} RM categories discovered; timesheet pickers may be incomplete",
            categories.len()
        );
    }

    let catalog_projects: Vec<RmProjectCatalogEntry> = projects
        .into_iter()
        .map(|project| RmProjectCatalogEntry {
            assignable_id: project.id,
            name: project.name,
            project_code: non_empty(project.project_code),
            phase_name: non_empty(project.phase_name),
            parent_project_id: project.parent_id,
            client: project_client_label(&project.client),
            categories: Vec::new(),
        })
        .collect();

    let fetched_at = Utc::now().to_rfc3339();
    let cache = RmCatalogCache {
        fetched_at: fetched_at.clone(),
        categories,
        projects: catalog_projects,
    };

    save_catalog(app, &cache)?;

    let mut settings = load_settings(app)?;
    settings.catalog_fetched_at = Some(fetched_at);
    save_settings(app, &settings)?;

    emit_progress(
        app,
        2,
        2,
        format!(
            "Cached {} projects and {} timesheet categories from Resource Management",
            cache.projects.len(),
            cache.categories.len()
        ),
    );

    Ok(cache)
}

fn discover_timesheet_categories(
    client: &RmClient,
    projects: &[RmProject],
    app: &AppHandle,
) -> Result<Vec<String>, String> {
    let mut seen = HashSet::new();
    let mut categories = Vec::new();

    for category in fetch_paginated_category_names(client, "time_entry_categories")? {
        push_category(&mut seen, &mut categories, category);
    }

    let probe_ids = build_probe_order(projects);
    let probe_count = probe_ids.len().min(MAX_CATEGORY_PROBE_PROJECTS);

    for (index, project_id) in probe_ids
        .into_iter()
        .take(MAX_CATEGORY_PROBE_PROJECTS)
        .enumerate()
    {
        let project_name = projects
            .iter()
            .find(|project| project.id == project_id)
            .map(|project| project.name.as_str())
            .unwrap_or("project");

        emit_progress(
            app,
            1,
            2,
            format!("Discovering categories from {project_name}"),
        );

        for category in fetch_project_time_entry_categories(client, project_id)? {
            push_category(&mut seen, &mut categories, category);
        }

        for category in fetch_project_budget_categories(client, project_id)? {
            push_category(&mut seen, &mut categories, category);
        }

        for category in fetch_project_task_categories(client, project_id)? {
            push_category(&mut seen, &mut categories, category);
        }

        if categories.len() >= MIN_USEFUL_CATEGORIES && index >= 2 {
            break;
        }

        if index + 1 < probe_count {
            thread::sleep(CATEGORY_PROBE_DELAY);
        }
    }

    categories.sort_by(|left, right| left.cmp(right));
    Ok(categories)
}

fn push_category(seen: &mut HashSet<String>, categories: &mut Vec<String>, value: String) {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return;
    }

    let key = trimmed.to_lowercase();
    if seen.insert(key) {
        categories.push(trimmed);
    }
}

fn build_probe_order(projects: &[RmProject]) -> Vec<i64> {
    let mut priority_ids = Vec::new();
    let mut rest_ids = Vec::new();

    for project in projects {
        let is_priority = PRIORITY_PROJECT_HINTS
            .iter()
            .any(|hint| project.name.contains(hint));

        if is_priority {
            priority_ids.push(project.id);
        } else {
            rest_ids.push(project.id);
        }
    }

    priority_ids.extend(rest_ids);
    priority_ids
}

fn fetch_paginated_category_names(
    client: &RmClient,
    path: &str,
) -> Result<Vec<String>, String> {
    let mut categories = Vec::new();
    let mut seen = HashSet::new();
    let mut page = 1_i64;

    loop {
        let page_str = page.to_string();
        let query = [("per_page", "1000"), ("page", page_str.as_str())];
        let response: RmPagedResponse<RmBudgetCategory> = client.get_json(path, &query)?;

        if response.data.is_empty() {
            break;
        }

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

    Ok(categories)
}

fn fetch_project_time_entry_categories(
    client: &RmClient,
    project_id: i64,
) -> Result<Vec<String>, String> {
    let path = format!("projects/{project_id}/time_entry_categories");
    let mut categories = Vec::new();
    let mut seen = HashSet::new();

    let query_sets: [&[(&str, &str)]; 3] = [
        &[("per_page", "1000"), ("page", "1")],
        &[
            ("per_page", "1000"),
            ("page", "1"),
            ("item_type", "TimeFees"),
        ],
        &[
            ("per_page", "1000"),
            ("page", "1"),
            ("item_type", "TimeFeesDays"),
        ],
    ];

    for query in query_sets {
        let response: RmPagedResponse<RmBudgetCategory> = match client.get_json(&path, query) {
            Ok(value) => value,
            Err(_) => continue,
        };

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
    }

    Ok(categories)
}

fn fetch_project_budget_categories(
    client: &RmClient,
    project_id: i64,
) -> Result<Vec<String>, String> {
    let path = format!("projects/{project_id}/budget_items");
    let mut categories = Vec::new();
    let mut seen = HashSet::new();

    for item_type in ["TimeFees", "TimeFeesDays"] {
        let query = [
            ("item_type", item_type),
            ("per_page", "1000"),
            ("page", "1"),
        ];

        let response: RmPagedResponse<RmBudgetItem> = match client.get_json(&path, &query) {
            Ok(value) => value,
            Err(_) => continue,
        };

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
    }

    Ok(categories)
}

fn fetch_project_task_categories(client: &RmClient, project_id: i64) -> Result<Vec<String>, String> {
    let path = format!("projects/{project_id}/time_entries");
    let query = [("per_page", "1000"), ("page", "1")];
    let response: RmPagedResponse<RmTimeEntry> = match client.get_json(&path, &query) {
        Ok(value) => value,
        Err(_) => return Ok(Vec::new()),
    };

    let mut categories = Vec::new();
    let mut seen = HashSet::new();

    for entry in response.data {
        if let Some(task) = entry.task {
            let trimmed = task.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            let key = trimmed.to_lowercase();
            if seen.insert(key) {
                categories.push(trimmed);
            }
        }
    }

    Ok(categories)
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
