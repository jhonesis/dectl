use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn dectl_bin() -> String {
    env!("CARGO_BIN_EXE_dectl").to_string()
}

fn run_dectl(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(dectl_bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("Failed to execute dectl")
}

#[test]
fn test_session_end_help() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["session", "end", "--help"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--skip-git"));
}

#[test]
fn test_session_end_dry_run() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["session", "end", "--dry-run"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Session ended") || stdout.contains("last_session.md"));
    assert!(!tmp.path().join(".dec/state/last_session.md").exists());
}

#[test]
fn test_session_end_skip_git() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["session", "end", "--skip-git"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("skipped") || stdout.contains("Session ended"));
}

#[test]
fn test_session_end_json() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["--json", "session", "end"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
    let data = parsed.get("data").expect("data field should exist");
    assert!(data.get("steps").is_some());
    assert!(data.get("decisions_saved").is_some());
}

#[test]
fn test_session_end_no_dec_directory() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["session", "end"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Session ended") || stdout.contains("last_session.md"));
}

#[test]
fn test_session_end_dry_run_with_skip_git() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["session", "end", "--dry-run", "--skip-git"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("skipped") || stdout.contains("Session ended"));
    assert!(!tmp.path().join(".dec/state/last_session.md").exists());
}

#[test]
fn test_session_end_json_dry_run() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["--json", "session", "end", "--dry-run"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_line = stdout
        .lines()
        .find(|l| l.starts_with('{'))
        .expect("Should contain JSON output");
    let parsed: serde_json::Value =
        serde_json::from_str(json_line).expect("Output should be valid JSON");
    let data = parsed.get("data").expect("data field should exist");
    let steps = data.get("steps").expect("steps should exist");
    assert!(steps.is_array());
}

#[test]
fn test_session_end_with_dec_initialized() {
    let tmp = TempDir::new().unwrap();
    run_dectl(&["project", "init"], tmp.path());
    let output = run_dectl(&["session", "end"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Session ended") || stdout.contains("last_session.md"));
    assert!(tmp.path().join(".dec/state/last_session.md").exists());
    let content = fs::read_to_string(tmp.path().join(".dec/state/last_session.md")).unwrap();
    assert!(!content.is_empty());
}
