use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub hash: String,
    pub message: String,
}

pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn recent_commits(count: usize) -> Result<Vec<CommitEntry>> {
    let output = Command::new("git")
        .args(["log", "--oneline", &format!("-{}", count)])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                Some(CommitEntry {
                    hash: parts[0].to_string(),
                    message: parts[1].to_string(),
                })
            } else {
                None
            }
        })
        .collect())
}

pub fn raw_commit_log(count: usize) -> Result<String> {
    let output = Command::new("git")
        .args(["log", "--oneline", &format!("-{}", count)])
        .output()?;

    if !output.status.success() {
        return Ok(String::new());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn diff_status() -> Result<Vec<(String, String)>> {
    let output = Command::new("git")
        .args(["diff", "--name-status"])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(tab_idx) = line.find('\t') {
            let status = line[..tab_idx].to_string();
            let path = line[tab_idx + 1..].to_string();
            files.push((status, path));
        }
    }
    Ok(files)
}

pub fn diff_since(rev: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", &format!("{rev}..HEAD")])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect())
}
