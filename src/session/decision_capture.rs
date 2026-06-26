use crate::core::db::Storage;
use crate::session::types::CapturedDecision;
use anyhow::Result;
use regex::Regex;
use rusqlite::params;
use std::fs;
use std::path::Path;

fn get_project_name() -> Option<String> {
    let config_path = Path::new(".dec/config/project.toml");
    if config_path.exists() {
        let content = fs::read_to_string(config_path).ok()?;
        let parsed: toml::Value = toml::from_str(&content).ok()?;
        parsed
            .get("project")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    } else {
        None
    }
}

fn load_existing_memories() -> Result<Vec<String>> {
    let db = crate::core::db::get_db()?;
    db.query_map(
        "SELECT content FROM memories WHERE deleted_at IS NULL",
        &[],
        |row| row.get(0),
    )
}

fn extract_decisions_from_text(text: &str) -> Vec<String> {
    let patterns = vec![
        r"(?i)vamos a usar\s+(.+?)(?:\.|\n|$)",
        r"(?i)decidimos que\s+(.+?)(?:\.|\n|$)",
        r"(?i)el stack será\s+(.+?)(?:\.|\n|$)",
        r"(?i)elegimos\s+(.+?)(?:\.|\n|$)",
        r"(?i)decidimos\s+(.+?)(?:\.|\n|$)",
        r"(?i)we decided (?:to )?(.+?)(?:\.|\n|$)",
        r"(?i)we'll use\s+(.+?)(?:\.|\n|$)",
        r"(?i)chose\s+(.+?)(?:\.|\n|$)",
        r"(?i)decided to\s+(.+?)(?:\.|\n|$)",
        r"(?i)going with\s+(.+?)(?:\.|\n|$)",
    ];

    let mut decisions = Vec::new();

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(text) {
                if let Some(m) = cap.get(1) {
                    let decision = m.as_str().trim().to_string();
                    if !decision.is_empty() && decision.len() > 5 {
                        decisions.push(decision);
                    }
                }
            }
        }
    }

    decisions.sort();
    decisions.dedup();
    decisions
}

pub fn capture_decisions() -> Result<Vec<CapturedDecision>> {
    let mut all_text = String::new();

    if Path::new(".dec/state/last_session.md").exists() {
        if let Ok(content) = fs::read_to_string(".dec/state/last_session.md") {
            all_text.push_str(&content);
            all_text.push('\n');
        }
    }

    if let Ok(git_log) = crate::core::git::raw_commit_log(20) {
        all_text.push_str(&git_log);
    }

    if all_text.is_empty() {
        return Ok(Vec::new());
    }

    let detected = extract_decisions_from_text(&all_text);
    let existing = load_existing_memories().unwrap_or_default();

    let decisions: Vec<CapturedDecision> = detected
        .into_iter()
        .map(|text| {
            let already_exists = existing.iter().any(|m| {
                let m_lower = m.to_lowercase();
                let text_lower = text.to_lowercase();
                m_lower.contains(&text_lower) || text_lower.contains(&m_lower)
            });

            CapturedDecision {
                text,
                tags: vec!["decision".to_string(), "session".to_string()],
                already_exists,
            }
        })
        .collect();

    Ok(decisions)
}

pub fn save_decisions(decisions: &[CapturedDecision]) -> Result<usize> {
    let db = crate::core::db::get_db()?;
    let project = get_project_name();
    let now = chrono::Utc::now().to_rfc3339();

    let mut count = 0;
    for decision in decisions {
        if decision.already_exists {
            continue;
        }

        let tags = decision.tags.join(",");
        let project_ref = project.as_deref();

        db.execute(
            "INSERT INTO memories (content, tags, project, created_at, updated_at, type) VALUES (?1, ?2, ?3, ?4, ?5, 'decision')",
            params![decision.text, tags, project_ref, now, now],
        )?;

        count += 1;
    }

    Ok(count)
}
