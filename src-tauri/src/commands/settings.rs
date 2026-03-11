use tauri::command;

use council_core::models::config::AppSettings;

#[command]
pub fn load_settings() -> Result<AppSettings, String> {
    council_core::settings::load_settings()
}

#[command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    council_core::settings::save_settings(&settings)
}
