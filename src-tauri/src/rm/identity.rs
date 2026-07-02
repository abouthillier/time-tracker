use chrono::Utc;

use super::client::RmClient;
use super::settings::{load_settings, save_settings};
use super::types::{RmPagedResponse, RmUser, RmUserCandidates, RmUserSummary};
use crate::rm::token;

pub fn find_user_candidates(token: &str, email: &str) -> Result<RmUserCandidates, String> {
    let trimmed_email = email.trim();
    if trimmed_email.is_empty() {
        return Err("Enter your work email to find your Resource Management user.".to_string());
    }

    let client = RmClient::new(token)?;
    let users = fetch_all_users(&client)?;
    let normalized = trimmed_email.to_lowercase();

    let candidates: Vec<RmUserSummary> = users
        .into_iter()
        .filter(|user| {
            user.email
                .as_ref()
                .map(|value| value.trim().to_lowercase() == normalized)
                .unwrap_or(false)
        })
        .map(user_summary)
        .collect();

    if candidates.is_empty() {
        return Err(format!(
            "No Resource Management user found for {trimmed_email}."
        ));
    }

    Ok(RmUserCandidates { candidates })
}

pub fn verify_linked_user(client: &RmClient, linked: &super::types::RmLinkedUser) -> Result<(), String> {
    let path = format!("users/{}", linked.id);
    let user: RmUser = client.get_json(&path, &[])?;

    let email = user
        .email
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_lowercase();

    if email != linked.email.trim().to_lowercase() {
        return Err(
            "Linked Resource Management user no longer matches the saved email. Unlink and link again."
                .to_string(),
        );
    }

    Ok(())
}

pub fn link_user(
    app: &tauri::AppHandle,
    user_id: i64,
    email: String,
    display_name: String,
) -> Result<super::types::RmLinkedUser, String> {
    let linked_user = super::types::RmLinkedUser {
        id: user_id,
        email: email.trim().to_string(),
        display_name: display_name.trim().to_string(),
        linked_at: Utc::now().to_rfc3339(),
    };

    let mut settings = load_settings(app)?;
    settings.linked_user = Some(linked_user.clone());
    save_settings(app, &settings)?;

    Ok(linked_user)
}

pub fn unlink_user(app: &tauri::AppHandle) -> Result<(), String> {
    let mut settings = load_settings(app)?;
    settings.linked_user = None;
    save_settings(app, &settings)
}

pub fn get_linked_user(app: &tauri::AppHandle) -> Result<Option<super::types::RmLinkedUser>, String> {
    let settings = load_settings(app)?;
    Ok(settings.linked_user)
}

fn fetch_all_users(client: &RmClient) -> Result<Vec<RmUser>, String> {
    let mut users = Vec::new();
    let mut page = 1_i64;

    loop {
        let page_str = page.to_string();
        let query = [("per_page", "1000"), ("page", page_str.as_str())];
        let response: RmPagedResponse<RmUser> = client.get_json("users", &query)?;

        if response.data.is_empty() {
            break;
        }

        users.extend(response.data);

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

    Ok(users)
}

fn user_summary(user: RmUser) -> RmUserSummary {
    let email = user.email.unwrap_or_default();
    let display_name = user
        .display_name
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            let first = user.first_name.unwrap_or_default();
            let last = user.last_name.unwrap_or_default();
            let combined = format!("{first} {last}").trim().to_string();
            if combined.is_empty() {
                None
            } else {
                Some(combined)
            }
        })
        .unwrap_or_else(|| email.clone());

    RmUserSummary {
        id: user.id,
        email,
        display_name,
    }
}

pub fn require_token() -> Result<String, String> {
    token::get_token()?.ok_or_else(|| {
        "No Resource Management API token saved. Add one in Resource Management settings.".to_string()
    })
}

pub fn load_settings_linked(app: &tauri::AppHandle) -> Result<super::types::RmLinkedUser, String> {
    let settings = load_settings(app)?;
    settings.linked_user.ok_or_else(|| {
        "Link your Resource Management user in settings before syncing.".to_string()
    })
}
