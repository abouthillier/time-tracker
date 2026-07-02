const SERVICE: &str = "com.alexb.time-tracker";
const TOKEN_ACCOUNT: &str = "rm-api-token";

pub fn save_token(token: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, TOKEN_ACCOUNT).map_err(|error| error.to_string())?;
    entry
        .set_password(token.trim())
        .map_err(|error| error.to_string())
}

pub fn get_token() -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(SERVICE, TOKEN_ACCOUNT).map_err(|error| error.to_string())?;

    match entry.get_password() {
        Ok(password) if password.trim().is_empty() => Ok(None),
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

pub fn clear_token() -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, TOKEN_ACCOUNT).map_err(|error| error.to_string())?;

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn has_token() -> Result<bool, String> {
    Ok(get_token()?.is_some())
}
