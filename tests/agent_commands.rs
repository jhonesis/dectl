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
fn test_agent_list_shows_builtins() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["agent", "list"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("coder"));
    assert!(stdout.contains("reviewer"));
    assert!(stdout.contains("researcher"));
    assert!(stdout.contains("documenter"));
}

#[test]
fn test_agent_list_json() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["--json", "agent", "list"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
    let data = parsed.get("data").expect("data field should exist");
    let agents = data.get("agents").expect("agents field should exist");
    assert!(agents.is_array());
    let names: Vec<&str> = agents
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.get("name").unwrap().as_str().unwrap())
        .collect();
    assert!(names.contains(&"coder"));
    assert!(names.contains(&"reviewer"));
    assert!(names.contains(&"researcher"));
    assert!(names.contains(&"documenter"));
}

#[test]
fn test_agent_describe_shows_full_definition() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["agent", "describe", "coder"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("coder"));
    assert!(stdout.contains("Feature implementer"));
    assert!(stdout.contains("Steps"));
    assert!(stdout.contains("[prompt]"));
    assert!(stdout.contains("[action]"));
}

#[test]
fn test_agent_describe_json() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["--json", "agent", "describe", "reviewer"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
    let data = parsed.get("data").expect("data field should exist");
    let agent = data.get("agent").expect("agent field should exist");
    assert_eq!(agent.get("name").unwrap().as_str().unwrap(), "reviewer");
    assert!(agent.get("steps").unwrap().is_array());
    assert!(agent.get("source").is_some());
}

#[test]
fn test_agent_describe_unknown_agent() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["agent", "describe", "nonexistent"], tmp.path());
    assert!(!output.status.success());
}

#[test]
fn test_agent_run_dry_run() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["agent", "run", "coder", "--task", "test", "--dry-run"],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[DRY-RUN]"));
    assert!(stdout.contains("Task: test"));
}

#[test]
fn test_agent_run_parallel_dry_run() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &[
            "agent",
            "run",
            "--parallel",
            "reviewer,documenter",
            "--task",
            "test",
            "--dry-run",
        ],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Running agents in parallel"));
    assert!(stdout.contains("reviewer"));
    assert!(stdout.contains("documenter"));
}

#[test]
fn test_agent_run_unknown_agent() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["agent", "run", "nonexistent", "--task", "test"],
        tmp.path(),
    );
    assert!(!output.status.success());
}

#[test]
fn test_agent_list_json_has_source() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["--json", "agent", "list"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");
    let data = parsed.get("data").expect("data field should exist");
    let agents = data.get("agents").unwrap().as_array().unwrap();
    for agent in agents {
        let source = agent.get("source").unwrap().as_str().unwrap();
        assert!(source == "builtin" || source.starts_with("custom"));
    }
}
