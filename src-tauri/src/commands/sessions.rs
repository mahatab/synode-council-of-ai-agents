use tauri::command;

use council_core::models::session::{Session, SessionSummary};

#[command]
pub fn save_session(session: Session, custom_path: Option<String>) -> Result<(), String> {
    council_core::sessions::save_session(&session, custom_path.as_deref())
}

#[command]
pub fn load_session(
    session_id: String,
    custom_path: Option<String>,
) -> Result<Session, String> {
    council_core::sessions::load_session(&session_id, custom_path.as_deref())
}

#[command]
pub fn list_sessions(custom_path: Option<String>) -> Result<Vec<SessionSummary>, String> {
    council_core::sessions::list_sessions(custom_path.as_deref())
}

#[command]
pub fn delete_session(session_id: String, custom_path: Option<String>) -> Result<(), String> {
    council_core::sessions::delete_session(&session_id, custom_path.as_deref())
}

#[command]
pub fn get_default_sessions_path() -> Result<String, String> {
    council_core::sessions::get_default_sessions_path()
}
