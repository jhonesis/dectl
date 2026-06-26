use anyhow::Result;

pub fn query_agent_sessions() -> Result<usize> {
    let last_session_path = std::path::Path::new(".dec/state/last_session.md");

    let timestamp = if last_session_path.exists() {
        let content = std::fs::read_to_string(last_session_path)?;
        extract_last_session_date(&content)
    } else {
        None
    };

    let db = crate::core::db::get_db()?;
    match timestamp {
        Some(ts) => crate::agent::log::query_agent_sessions_since(db, &ts),
        None => Ok(0),
    }
}

fn extract_last_session_date(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(date_str) = trimmed.strip_prefix("**Fecha**:") {
            return Some(date_str.trim().to_string());
        }
        if let Some(date_str) = trimmed.strip_prefix("**Date**:") {
            return Some(date_str.trim().to_string());
        }
    }
    None
}
