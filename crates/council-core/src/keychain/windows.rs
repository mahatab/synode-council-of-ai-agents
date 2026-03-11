use keyring::Entry;
use std::collections::HashMap;

use super::{KEYCHAIN_ACCOUNT, KEYCHAIN_SERVICE};

/// Read the single JSON blob from Windows Credential Manager and parse into a HashMap.
pub fn read_keychain_blob() -> HashMap<String, String> {
    match Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
        Ok(entry) => match entry.get_password() {
            Ok(json_str) => serde_json::from_str(&json_str).unwrap_or_default(),
            Err(_) => HashMap::new(),
        },
        Err(_) => HashMap::new(),
    }
}

/// Write the full HashMap as a JSON blob to Windows Credential Manager.
pub fn write_keychain_blob(keys: &HashMap<String, String>) -> Result<(), String> {
    let json = serde_json::to_string(keys)
        .map_err(|e| format!("Failed to serialize API keys: {}", e))?;

    let entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .map_err(|e| format!("Failed to create credential entry: {}", e))?;

    entry
        .set_password(&json)
        .map_err(|e| format!("Failed to save API keys to Credential Manager: {}", e))
}

/// No legacy migration needed on Windows — there are no old per-provider entries.
pub fn migrate_legacy_keys() -> HashMap<String, String> {
    HashMap::new()
}
