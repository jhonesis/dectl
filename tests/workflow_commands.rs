use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[derive(serde::Deserialize)]
struct Workflow {
    steps: Vec<Step>,
}

#[derive(serde::Deserialize)]
struct Step {
    #[serde(rename = "type")]
    step_type: String,
    content: Option<String>,
    run_always: Option<bool>,
}

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
    assert!(stdout.contains("execute_task"));
    assert!(stdout.contains("[5]"));
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
    assert!(stdout.contains("Step 5"));
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

#[test]
fn execute_task_workflow_has_run_always_on_steps_4_and_5() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);
    let workflow_path = tmp.path().join(".dec/workflows/execute_task.yaml");
    let content = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", workflow_path.display(), e));
    let workflow: Workflow =
        serde_yaml::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse YAML: {}", e));

    assert_eq!(workflow.steps.len(), 5, "Workflow should have 5 steps");

    let step3 = &workflow.steps[2];
    assert_eq!(step3.step_type, "prompt", "Step 3 should be a prompt");
    let step3_content = step3.content.as_ref().expect("Step 3 should have content");
    assert!(
        step3_content.contains("--from-step 4"),
        "Step 3 content must contain '--from-step 4' command"
    );

    let step4 = &workflow.steps[3];
    assert_eq!(step4.step_type, "agent", "Step 4 should be an agent");
    assert_eq!(
        step4.run_always,
        Some(true),
        "Step 4 (reviewer) must have run_always: true"
    );

    let step5 = &workflow.steps[4];
    assert_eq!(step5.step_type, "agent", "Step 5 should be an agent");
    assert_eq!(
        step5.run_always,
        Some(true),
        "Step 5 (documenter) must have run_always: true"
    );
}
