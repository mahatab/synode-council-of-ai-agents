use std::fs;
use std::path::PathBuf;

use crate::models::session::{Session, SessionSummary};

fn get_sessions_dir(custom_path: Option<&str>) -> PathBuf {
    if let Some(path) = custom_path {
        PathBuf::from(path)
    } else {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("council-of-ai-agents")
            .join("sessions")
    }
}

fn ensure_sessions_dir(custom_path: Option<&str>) -> Result<PathBuf, String> {
    let dir = get_sessions_dir(custom_path);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create sessions directory: {}", e))?;
    Ok(dir)
}

pub fn save_session(session: &Session, custom_path: Option<&str>) -> Result<(), String> {
    let dir = ensure_sessions_dir(custom_path)?;
    let file_path = dir.join(format!("{}.json", session.id));
    let json = serde_json::to_string_pretty(session)
        .map_err(|e| format!("Serialization error: {}", e))?;
    fs::write(file_path, json).map_err(|e| format!("Failed to write session file: {}", e))
}

pub fn load_session(session_id: &str, custom_path: Option<&str>) -> Result<Session, String> {
    let dir = get_sessions_dir(custom_path);
    let file_path = dir.join(format!("{}.json", session_id));
    let json =
        fs::read_to_string(&file_path).map_err(|e| format!("Failed to read session file: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse session: {}", e))
}

pub fn list_sessions(custom_path: Option<&str>) -> Result<Vec<SessionSummary>, String> {
    let dir = get_sessions_dir(custom_path);

    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions: Vec<SessionSummary> = Vec::new();

    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read sessions directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(session) = serde_json::from_str::<Session>(&json) {
                    sessions.push(SessionSummary {
                        id: session.id,
                        title: session.title,
                        created_at: session.created_at,
                        updated_at: session.updated_at,
                        session_type: session.session_type,
                    });
                }
            }
        }
    }

    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(sessions)
}

pub fn delete_session(session_id: &str, custom_path: Option<&str>) -> Result<(), String> {
    let dir = get_sessions_dir(custom_path);
    let file_path = dir.join(format!("{}.json", session_id));

    if file_path.exists() {
        fs::remove_file(file_path).map_err(|e| format!("Failed to delete session: {}", e))
    } else {
        Ok(())
    }
}

pub fn get_default_sessions_path() -> Result<String, String> {
    let path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("council-of-ai-agents")
        .join("sessions");
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to convert path to string".to_string())
}
