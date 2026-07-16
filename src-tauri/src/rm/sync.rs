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

fn put_update_body(payload: &EntryPayload) -> serde_json::Value {
    serde_json::json!({
        "hours": payload.hours,
        "task": payload.category,
        "notes": if payload.notes.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(payload.notes.clone()) },
    })
}

fn put_entry(
    client: &RmClient,
    linked_user_id: i64,
    remote_id: i64,
    _entry: &TimeEntry,
    payload: &EntryPayload,
) -> Result<i64, String> {
    let path = format!("users/{linked_user_id}/time_entries/{remote_id}");
    // RM rejects PUT bodies that try to set identity fields (assignable_id, user_id,
    // date) on confirmed entries — only hours/task/notes are mutable.
    let body = put_update_body(payload);

    let updated: RmCreatedTimeEntry = client.request_json("PUT", &path, &[], Some(body))?;
    Ok(updated.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entries::{RmEntrySync, TimeEntry, TimeSlot};
    use super::super::types::{RmCatalogCache, RmProjectCatalogEntry, SyncScope};

    fn sample_entry(status: &str, remote_id: Option<i64>, synced_hash: Option<&str>) -> TimeEntry {
        TimeEntry {
            id: "entry-1".to_string(),
            date: "2026-07-01".to_string(),
            project: "ELC 2025".to_string(),
            assignable_id: Some(42),
            category: Some("Admin".to_string()),
            entries: vec![TimeSlot {
                start_time: "09:00".to_string(),
                end_time: "11:30".to_string(),
                notes: Some("notes".to_string()),
            }],
            rm_sync: Some(RmEntrySync {
                remote_id,
                last_synced_at: None,
                synced_hash: synced_hash.map(str::to_string),
                status: status.to_string(),
                last_error: None,
                last_error_at: None,
                last_attempt_at: None,
            }),
        }
    }

    #[test]
    fn build_payload_aggregates_hours_and_hash() {
        let entry = sample_entry("pending", None, None);
        let payload = build_payload(&entry).expect("payload");

        assert_eq!(payload.hours, 2.5);
        assert_eq!(payload.category, "Admin");
        assert_eq!(payload.notes, "notes");
        assert_eq!(payload.hash, "2026-07-01|42|Admin|2.50|notes");
    }

    #[test]
    fn should_attempt_changed_skips_synced_with_matching_hash() {
        let entry = sample_entry("synced", Some(99), Some("2026-07-01|42|Admin|2.50|notes"));
        let payload = build_payload(&entry).expect("payload");
        let selected = std::collections::HashSet::new();

        assert!(!should_attempt(
            &entry,
            &SyncScope::Changed,
            &payload.hash,
            &selected
        ));
    }

    #[test]
    fn should_attempt_selected_includes_explicit_entry() {
        let entry = sample_entry("synced", Some(99), Some("old-hash"));
        let mut selected = std::collections::HashSet::new();
        selected.insert("entry-1".to_string());

        assert!(should_attempt(&entry, &SyncScope::Selected, "any", &selected));
    }

    #[test]
    fn validate_against_catalog_rejects_unknown_category() {
        let entry = sample_entry("pending", None, None);
        let payload = build_payload(&entry).expect("payload");
        let catalog = RmCatalogCache {
            fetched_at: "2026-07-01T00:00:00Z".to_string(),
            categories: vec!["3D modeling".to_string()],
            projects: vec![RmProjectCatalogEntry {
                assignable_id: 42,
                name: "ELC 2025".to_string(),
                project_code: None,
                phase_name: None,
                parent_project_id: None,
                client: None,
                categories: vec![],
            }],
        };

        let error = validate_against_catalog(&entry, &catalog, &payload).unwrap_err();
        assert!(error.contains("Admin"));
    }

    #[test]
    fn put_update_body_excludes_immutable_identity_fields() {
        let body = put_update_body(&EntryPayload {
            assignable_id: 10954953,
            category: "Programming".to_string(),
            hours: 2.0,
            notes: String::new(),
            hash: String::new(),
        });

        assert!(body.get("assignable_id").is_none());
        assert!(body.get("user_id").is_none());
        assert!(body.get("date").is_none());
        assert_eq!(body["hours"], 2.0);
        assert_eq!(body["task"], "Programming");
    }
}

