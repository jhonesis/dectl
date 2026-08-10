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

/// Run dectl with the test binary's directory prepended to PATH so nested
/// `dectl` subprocess calls inside agent action steps resolve correctly.
fn run_dectl_with_path(args: &[&str], cwd: &Path) -> std::process::Output {
    let bin_path = dectl_bin();
    let bin_dir = std::path::Path::new(&bin_path).parent().unwrap();
    let mut env = std::env::vars().collect::<std::collections::HashMap<_, _>>();
    if let Some(current_path) = env.get("PATH") {
        let new_path = format!("{}:{}", bin_dir.display(), current_path);
        env.insert("PATH".to_string(), new_path);
    }
    let mut cmd = Command::new(&bin_path);
    cmd.args(args).current_dir(cwd);
    for (key, val) in env.iter() {
        cmd.env(key, val);
    }
    cmd.output().expect("Failed to execute dectl")
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
    assert!(stdout.contains("feature implementer"));
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
        &[
            "agent",
            "run",
            "coder",
            "--task",
            "test",
            "--var",
            "task_id=test",
            "--dry-run",
        ],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[DRY-RUN]"));
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
            "--var",
            "task_id=test",
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

#[test]
fn test_agent_trust_builtin() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["agent", "trust", "researcher"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("researcher"));
    assert!(stdout.contains("trusted"));
}

#[test]
fn test_agent_trust_nonexistent() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["agent", "trust", "nonexistent"], tmp.path());
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"));
}

#[test]
fn test_agent_trust_invalid_path() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &[
            "agent",
            "trust",
            "researcher",
            "--project",
            "/nonexistent/path/xxxxx",
        ],
        tmp.path(),
    );
    assert!(!output.status.success());
}

#[test]
fn test_agent_run_non_interactive_error_suggests_trust() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &[
            "agent",
            "run",
            "researcher",
            "--task",
            "test",
            "--var",
            "task_id=test",
            "--non-interactive",
        ],
        tmp.path(),
    );
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("dectl agent trust"));
    assert!(combined.contains("researcher"));
}

#[test]
fn test_agent_trust_then_dry_run() {
    let tmp = TempDir::new().unwrap();
    let trust_output = run_dectl(&["agent", "trust", "researcher"], tmp.path());
    assert!(trust_output.status.success());

    let run_output = run_dectl(
        &[
            "agent",
            "run",
            "researcher",
            "--task",
            "test",
            "--var",
            "task_id=test",
            "--dry-run",
        ],
        tmp.path(),
    );
    assert!(run_output.status.success());
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("[DRY-RUN]"));
}

#[test]
fn test_agent_run_auto_skips_trust() {
    let tmp = TempDir::new().unwrap();
    // The coder agent contains action steps, so without --auto it would
    // require trust confirmation. With --auto the trust check must be skipped.
    let output = run_dectl_with_path(
        &[
            "agent",
            "run",
            "coder",
            "--task",
            "t031 auto test",
            "--var",
            "task_id=t031-auto",
            "--auto",
        ],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "agent run --auto should succeed without trust: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        !combined.contains("Do you trust"),
        "agent run --auto should not show the trust prompt, got:\n{}",
        combined
    );
    assert!(
        !combined.contains("is not trusted"),
        "agent run --auto should not fail with a trust error, got:\n{}",
        combined
    );
}
