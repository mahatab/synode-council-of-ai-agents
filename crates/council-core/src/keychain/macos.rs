use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};
use std::collections::HashMap;

use super::{KEYCHAIN_ACCOUNT, KEYCHAIN_SERVICE, LEGACY_ACCOUNT, LEGACY_SERVICES};

/// Read the single JSON blob from macOS Keychain and parse into a HashMap.
pub fn read_keychain_blob() -> HashMap<String, String> {
    match get_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
        Ok(bytes) => {
            let json_str = String::from_utf8(bytes.to_vec()).unwrap_or_default();
            serde_json::from_str(&json_str).unwrap_or_default()
        }
        Err(_) => HashMap::new(),
    }
}

/// Write the full HashMap as a JSON blob to the single macOS Keychain entry.
pub fn write_keychain_blob(keys: &HashMap<String, String>) -> Result<(), String> {
    let json = serde_json::to_string(keys)
        .map_err(|e| format!("Failed to serialize API keys: {}", e))?;

    // Delete existing entry first to avoid duplicates
    let _ = delete_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);

    set_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT, json.as_bytes())
        .map_err(|e| format!("Failed to save API keys to Keychain: {}", e))
}

/// Migrate old per-provider keychain entries into the new single entry.
/// Returns any keys found in the legacy entries.
pub fn migrate_legacy_keys() -> HashMap<String, String> {
    let mut migrated = HashMap::new();

    for (provider, service) in LEGACY_SERVICES {
        if let Ok(bytes) = get_generic_password(service, LEGACY_ACCOUNT) {
            if let Ok(key) = String::from_utf8(bytes.to_vec()) {
                if !key.is_empty() {
                    migrated.insert(provider.to_string(), key);
                }
            }
            // Clean up old entry
            let _ = delete_generic_password(service, LEGACY_ACCOUNT);
        }
    }

    migrated
}
