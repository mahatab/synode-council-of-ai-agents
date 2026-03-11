use tauri::{command, State};

use council_core::keychain::ApiKeyCache;

#[command]
pub fn save_api_key(
    service: String,
    api_key: String,
    cache: State<ApiKeyCache>,
) -> Result<(), String> {
    council_core::keychain::save_api_key(&cache, &service, &api_key)
}

#[command]
pub fn get_api_key(service: String, cache: State<ApiKeyCache>) -> Result<Option<String>, String> {
    council_core::keychain::get_api_key(&cache, &service)
}

#[command]
pub fn delete_api_key(service: String, cache: State<ApiKeyCache>) -> Result<(), String> {
    council_core::keychain::delete_api_key(&cache, &service)
}

#[command]
pub fn has_api_key(service: String, cache: State<ApiKeyCache>) -> Result<bool, String> {
    council_core::keychain::has_api_key(&cache, &service)
}
