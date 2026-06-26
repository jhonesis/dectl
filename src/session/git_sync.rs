use anyhow::Result;
use serde_json;
use std::fs;
use std::path::Path;

use crate::session::types::GitChanges;

pub fn detect_git_changes() -> Result<Option<GitChanges>> {
    if !crate::core::git::is_git_repo() {
        return Ok(None);
    }

    let modified_files = crate::core::git::diff_status()?;
    let new_commits: Vec<String> = crate::core::git::recent_commits(20)?
        .into_iter()
        .map(|c| format!("{} {}", c.hash, c.message))
        .collect();
    let detected_features = detect_features_from_commits(&new_commits);

    Ok(Some(GitChanges {
        modified_files,
        new_commits,
        detected_features,
    }))
}

fn detect_features_from_commits(commits: &[String]) -> Vec<String> {
    let mut features = Vec::new();

    for commit in commits {
        let lower = commit.to_lowercase();
        for pattern in &["feat:", "feature:", "add:"] {
            if let Some(idx) = lower.find(pattern) {
                let start = idx + pattern.len();
                if start < commit.len() {
                    let feature_text = commit[start..].trim().to_string();
                    if !feature_text.is_empty() && !features.contains(&feature_text) {
                        features.push(feature_text);
                    }
                }
            }
        }
    }

    features
}

pub fn sync_progress(git_changes: &GitChanges, dry_run: bool) -> Result<()> {
    let progress_path = Path::new(".dec/state/progress.json");
    let mut progress: serde_json::Value = if progress_path.exists() {
        let content = fs::read_to_string(progress_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({
            "updated_at": "",
            "features": []
        }))
    } else {
        serde_json::json!({
            "updated_at": "",
            "features": []
        })
    };

    if progress.get("features").is_none() {
        progress["features"] = serde_json::json!([]);
    }

    let modified_paths: Vec<&str> = git_changes
        .modified_files
        .iter()
        .map(|(_, path)| path.as_str())
        .collect();

    if let Some(features) = progress["features"].as_array_mut() {
        for feature in features.iter_mut() {
            if let Some(feature_id) = feature.get("id").and_then(|v| v.as_str()) {
                if let Some(status) = feature.get("status").and_then(|v| v.as_str()) {
                    if status == "in_progress" {
                        let related = modified_paths.iter().any(|p| p.contains(feature_id));
                        if related {
                            if dry_run {
                                println!("[dry-run] Would mark feature '{}' as 'done'", feature_id);
                            } else {
                                feature["status"] = serde_json::json!("done");
                            }
                        }
                    }
                }
            }
        }
    }

    if !git_changes.detected_features.is_empty() {
        if let Some(features) = progress["features"].as_array_mut() {
            let existing_ids: Vec<String> = features
                .iter()
                .filter_map(|f| f.get("id").and_then(|v| v.as_str()))
                .map(|s| s.to_string())
                .collect();

            for detected in &git_changes.detected_features {
                let feature_id = detected
                    .to_lowercase()
                    .split_whitespace()
                    .next()
                    .unwrap_or(detected)
                    .to_string();

                if !existing_ids.contains(&feature_id) {
                    if dry_run {
                        println!("[dry-run] Would add new feature '{}' from commit", detected);
                    } else {
                        features.push(serde_json::json!({
                            "id": feature_id,
                            "name": detected,
                            "status": "in_progress",
                            "notes": ""
                        }));
                    }
                }
            }
        }
    }

    progress["updated_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());

    if dry_run {
        println!("[dry-run] progress.json would be updated");
        println!("{}", serde_json::to_string_pretty(&progress)?);
    } else {
        if let Some(parent) = progress_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&progress)?;
        fs::write(progress_path, content)?;
    }

    Ok(())
}
