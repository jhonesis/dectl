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

fn init_project(tmp: &TempDir) {
    let output = run_dectl(&["project", "init", "--standard"], tmp.path());
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_execute_task_workflow_exists() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);
    let workflow_path = tmp.path().join(".dec/workflows/execute_task.yaml");
    assert!(
        workflow_path.exists(),
        "execute_task.yaml should exist after init --standard"
    );
}

#[test]
fn test_execute_task_workflow_describe() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);
    let output = run_dectl(&["workflow", "describe", "execute_task"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execute-task"));
    assert!(stdout.contains("[6]"));
}

#[test]
fn test_workflow_run_auto_flag() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);
    let output = run_dectl(
        &[
            "workflow",
            "run",
            "execute_task",
            "--dry-run",
            "--auto",
            "--var",
            "task_id=T001",
            "--var",
            "description=test task",
        ],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "auto dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Step 1"));
    assert!(stdout.contains("Step 6"));
}

#[test]
fn test_workflow_auto_skips_trust_prompt() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);
    let output = run_dectl(
        &[
            "workflow",
            "run",
            "execute_task",
            "--dry-run",
            "--auto",
            "--var",
            "task_id=T001",
            "--var",
            "description=test task",
        ],
        tmp.path(),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stdout.contains("Do you trust"),
        "Should not show trust prompt with --auto"
    );
    assert!(
        !stderr.contains("Do you trust"),
        "Should not show trust prompt with --auto"
    );
}
