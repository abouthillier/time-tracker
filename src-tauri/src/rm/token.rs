use std::sync::{Mutex, OnceLock};

const SERVICE: &str = "com.alexb.time-tracker";
const TOKEN_ACCOUNT: &str = "rm-api-token";

fn session_token() -> &'static Mutex<Option<String>> {
    static STORE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(None))
}

fn set_session_token(token: Option<String>) {
    *session_token().lock().expect("session token lock") = token;
}

pub fn save_token(token: &str) -> Result<(), String> {
    let trimmed = token.trim().to_string();
    if trimmed.is_empty() {
        return Err("API token cannot be empty.".to_string());
    }

    let entry = keyring::Entry::new(SERVICE, TOKEN_ACCOUNT).map_err(|error| error.to_string())?;
    entry
        .set_password(&trimmed)
        .map_err(|error| error.to_string())?;

    set_session_token(Some(trimmed));
    Ok(())
}

pub fn get_token() -> Result<Option<String>, String> {
    if let Some(cached) = session_token().lock().expect("session token lock").clone() {
        if !cached.trim().is_empty() {
            return Ok(Some(cached));
        }
    }

    let entry = keyring::Entry::new(SERVICE, TOKEN_ACCOUNT).map_err(|error| error.to_string())?;

    match entry.get_password() {
        Ok(password) if password.trim().is_empty() => Ok(None),
        Ok(password) => {
            set_session_token(Some(password.clone()));
            Ok(Some(password))
        }
        Err(keyring::Error::NoEntry) => Ok(session_token().lock().expect("session token lock").clone()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn clear_token() -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, TOKEN_ACCOUNT).map_err(|error| error.to_string())?;

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {
            set_session_token(None);
            Ok(())
        }
        Err(error) => Err(error.to_string()),
    }
}

pub fn has_token() -> Result<bool, String> {
    Ok(get_token()?.is_some())
}
