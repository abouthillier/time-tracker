use chrono::Utc;

use super::client::RmClient;
use super::identity::{load_settings_linked, require_token, verify_linked_user};
use super::settings::load_catalog;
use super::types::{
    RmCreatedTimeEntry, SyncDayRequest, SyncDayResult, SyncFailure, SyncPreviewRow, SyncScope,
};
use crate::entries::{save_entries, TimeEntry, TimeSlot};

pub fn sync_day(app: &tauri::AppHandle, request: SyncDayRequest) -> Result<SyncDayResult, String> {
    let token = require_token()?;
    let client = RmClient::new(&token)?;
    let linked_user = load_settings_linked(app)?;
    verify_linked_user(&client, &linked_user)?;

    let catalog = load_catalog(app)?.ok_or_else(|| {
        "Refresh the Resource Management catalog before syncing.".to_string()
    })?;

    let mut entries = crate::entries::load_entries(app)?;
    let exclude: std::collections::HashSet<String> = request
        .exclude_entry_ids
        .unwrap_or_default()
        .into_iter()
        .collect();
    let selected: std::collections::HashSet<String> = request
        .entry_ids
        .unwrap_or_default()
        .into_iter()
        .collect();

    let mut preview_rows = Vec::new();
    let mut succeeded = 0usize;
    let mut skipped = 0usize;
    let mut failed = Vec::new();

    let day_indices: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.date == request.date)
        .map(|(index, _)| index)
        .collect();

    for index in day_indices {
        let entry = &entries[index];
        let entry_id = entry.id.clone();

        if exclude.contains(&entry_id) {
            skipped += 1;
            continue;
        }

        let Some(payload) = build_payload(entry) else {
            if should_attempt(entry, &request.scope, "", &selected) {
                failed.push(SyncFailure {
                    entry_id: entry_id.clone(),
                    date: entry.date.clone(),
                    project: entry.project.clone(),
                    category: entry.category.clone().unwrap_or_default(),
                    error: "Entry is missing an RM project or category.".to_string(),
                    recoverable: true,
                });
            } else {
                skipped += 1;
            }
            continue;
        };

        if let Err(validation_error) = validate_against_catalog(entry, &catalog, &payload) {
            if should_attempt(entry, &request.scope, &payload.hash, &selected) {
                failed.push(SyncFailure {
                    entry_id: entry_id.clone(),
                    date: entry.date.clone(),
                    project: entry.project.clone(),
                    category: payload.category.clone(),
                    error: validation_error,
                    recoverable: true,
                });
            } else {
                skipped += 1;
            }
            continue;
        }

        if !should_attempt(entry, &request.scope, &payload.hash, &selected) {
            skipped += 1;
            continue;
        }

        let action = if entry.rm_sync.as_ref().and_then(|sync| sync.remote_id).is_some() {
            "PUT"
        } else {
            "POST"
        };

        if request.dry_run {
            preview_rows.push(SyncPreviewRow {
                entry_id: entry_id.clone(),
                project: entry.project.clone(),
                category: payload.category.clone(),
                hours: payload.hours,
                action: action.to_string(),
                hash: payload.hash.clone(),
                included: true,
            });
            continue;
        }

        let now = Utc::now().to_rfc3339();
        let entry_snapshot = entry.clone();
        let attempt_result = push_entry(&client, linked_user.id, &entry_snapshot, &payload);

        match attempt_result {
            Ok(remote_id) => {
                entries[index].rm_sync = Some(crate::entries::RmEntrySync {
                    remote_id: Some(remote_id),
                    last_synced_at: Some(now.clone()),
                    synced_hash: Some(payload.hash.clone()),
                    status: "synced".to_string(),
                    last_error: None,
                    last_error_at: None,
                    last_attempt_at: Some(now),
                });
                save_entries(app, &entries)?;
                succeeded += 1;
            }
            Err(error) => {
                entries[index].rm_sync = Some(crate::entries::RmEntrySync {
                    remote_id: entry_snapshot
                        .rm_sync
                        .as_ref()
                        .and_then(|sync| sync.remote_id),
                    last_synced_at: entry_snapshot
                        .rm_sync
                        .as_ref()
                        .and_then(|sync| sync.last_synced_at.clone()),
                    synced_hash: entry_snapshot
                        .rm_sync
                        .as_ref()
                        .and_then(|sync| sync.synced_hash.clone()),
                    status: "error".to_string(),
                    last_error: Some(error.clone()),
                    last_error_at: Some(now.clone()),
                    last_attempt_at: Some(now),
                });
                save_entries(app, &entries)?;
                failed.push(SyncFailure {
                    entry_id,
                    date: entry_snapshot.date,
                    project: entry_snapshot.project,
                    category: payload.category,
                    error,
                    recoverable: true,
                });
            }
        }
    }

    if request.dry_run {
        return Ok(SyncDayResult {
            succeeded: 0,
            skipped,
            failed,
            preview: Some(preview_rows),
            linked_user: Some(linked_user),
        });
    }

    Ok(SyncDayResult {
        succeeded,
        skipped,
        failed,
        preview: None,
        linked_user: Some(linked_user),
    })
}

struct EntryPayload {
    assignable_id: i64,
    category: String,
    hours: f64,
    notes: String,
    hash: String,
}

fn build_payload(entry: &TimeEntry) -> Option<EntryPayload> {
    let assignable_id = entry.assignable_id?;
    let category = entry.category.as_ref()?.trim().to_string();
    if category.is_empty() {
        return None;
    }

    let total_minutes: i64 = entry
        .entries
        .iter()
        .map(|slot| slot_duration_minutes(slot))
        .sum();

    let hours = ((total_minutes as f64 / 60.0) * 100.0).round() / 100.0;
    let notes = entry
        .entries
        .iter()
        .filter_map(|slot| slot.notes.as_ref())
        .map(|note| note.trim())
        .filter(|note| !note.is_empty())
        .collect::<Vec<_>>()
        .join("; ");
    let notes = if notes.chars().count() > 256 {
        notes.chars().take(256).collect()
    } else {
        notes
    };

    let hash = format!(
        "{}|{}|{}|{hours:.2}|{notes}",
        entry.date, assignable_id, category
    );

    Some(EntryPayload {
        assignable_id,
        category,
        hours,
        notes,
        hash,
    })
}

fn slot_duration_minutes(slot: &TimeSlot) -> i64 {
    let start = time_to_minutes(&slot.start_time);
    let end = time_to_minutes(&slot.end_time);
    (end - start).max(0)
}

fn time_to_minutes(time: &str) -> i64 {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return 0;
    }

    let hours = parts[0].parse::<i64>().unwrap_or(0);
    let minutes = parts[1].parse::<i64>().unwrap_or(0);
    hours * 60 + minutes
}

fn validate_against_catalog(
    entry: &TimeEntry,
    catalog: &super::types::RmCatalogCache,
    payload: &EntryPayload,
) -> Result<(), String> {
    let project_exists = catalog
        .projects
        .iter()
        .any(|project| project.assignable_id == entry.assignable_id.unwrap_or(-1));

    if !project_exists {
        return Err(format!(
            "Project \"{}\" is not in the cached catalog. Refresh the catalog and try again.",
            entry.project
        ));
    }

    if !catalog.categories.is_empty()
        && !catalog
            .categories
            .iter()
            .any(|category| category == &payload.category)
    {
        return Err(format!(
            "Category \"{}\" is not in the cached catalog. Refresh the catalog or pick another category.",
            payload.category
        ));
    }

    Ok(())
}

fn should_attempt(
    entry: &TimeEntry,
    scope: &SyncScope,
    content_hash: &str,
    selected: &std::collections::HashSet<String>,
) -> bool {
    let status = entry
        .rm_sync
        .as_ref()
        .map(|sync| sync.status.as_str())
        .unwrap_or("pending");

    let hash_matches = entry
        .rm_sync
        .as_ref()
        .and_then(|sync| sync.synced_hash.as_ref())
        .is_some_and(|hash| hash == content_hash);

    match scope {
        SyncScope::Failed => status == "error",
        SyncScope::Selected => selected.contains(&entry.id),
        SyncScope::All => !(status == "synced" && hash_matches),
        SyncScope::Changed => {
            matches!(status, "pending" | "dirty" | "error") || (status == "synced" && !hash_matches)
        }
    }
}

fn push_entry(
    client: &RmClient,
    linked_user_id: i64,
    entry: &TimeEntry,
    payload: &EntryPayload,
) -> Result<i64, String> {
    if let Some(remote_id) = entry.rm_sync.as_ref().and_then(|sync| sync.remote_id) {
        match put_entry(client, linked_user_id, remote_id, entry, payload) {
            Ok(id) => return Ok(id),
            Err(error) if error.contains("(404)") => {
                return post_entry(client, linked_user_id, entry, payload);
            }
            Err(error) => return Err(error),
        }
    }

    post_entry(client, linked_user_id, entry, payload)
}

fn post_entry(
    client: &RmClient,
    linked_user_id: i64,
    entry: &TimeEntry,
    payload: &EntryPayload,
) -> Result<i64, String> {
    let path = format!("users/{linked_user_id}/time_entries");
    let body = serde_json::json!({
        "user_id": linked_user_id,
        "assignable_id": payload.assignable_id,
        "date": entry.date,
        "hours": payload.hours,
        "task": payload.category,
        "notes": if payload.notes.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(payload.notes.clone()) },
    });

    let created: RmCreatedTimeEntry = client.request_json("POST", &path, &[], Some(body))?;
    Ok(created.id)
}

fn put_entry(
    client: &RmClient,
    linked_user_id: i64,
    remote_id: i64,
    entry: &TimeEntry,
    payload: &EntryPayload,
) -> Result<i64, String> {
    let path = format!("users/{linked_user_id}/time_entries/{remote_id}");
    let body = serde_json::json!({
        "user_id": linked_user_id,
        "assignable_id": payload.assignable_id,
        "date": entry.date,
        "hours": payload.hours,
        "task": payload.category,
        "notes": if payload.notes.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(payload.notes.clone()) },
    });

    let updated: RmCreatedTimeEntry = client.request_json("PUT", &path, &[], Some(body))?;
    Ok(updated.id)
}
