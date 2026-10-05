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
    cmd: Option<Vec<String>>,
    run_always: Option<bool>,
}

fn run_state_gate(
    tmp: &TempDir,
    review_status: &str,
    progress_status: &str,
    task_checkbox: &str,
    reviewer_result: &str,
) -> std::process::Output {
    init_project(tmp);
    let state_dir = tmp.path().join(".dec/state");
    let output_dir = tmp.path().join(".dec/agent-output");
    let specs_dir = tmp.path().join("specs");
    std::fs::create_dir_all(&output_dir).unwrap();
    std::fs::create_dir_all(&specs_dir).unwrap();
    std::fs::write(
        state_dir.join("progress.json"),
        format!(
            "{{\"features\":[{{\"id\":\"T057\",\"status\":\"{}\"}}]}}",
            progress_status
        ),
    )
    .unwrap();
    std::fs::write(
        state_dir.join("last_session.md"),
        "# Session\n\nT057 review attempt recorded\n",
    )
    .unwrap();
    std::fs::write(
        output_dir.join("T057-review-status.txt"),
        format!("{}\n", review_status),
    )
    .unwrap();
    std::fs::write(
        output_dir.join("T057-review.md"),
        "# Review attempt\n\nFixture report retained by the workflow gate.\n",
    )
    .unwrap();
    std::fs::write(
        specs_dir.join("tasks.md"),
        format!("- [{}] [T057] Example task\n", task_checkbox),
    )
    .unwrap();

    run_dectl(
        &[
            "workflow",
            "run",
            "execute_task",
            "--from-step",
            "7",
            "--auto",
            "--var",
            "task_id=T057",
            "--var",
            "description=Example task",
            "--var",
            &format!("step_4_output={reviewer_result}"),
        ],
        tmp.path(),
    )
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

    assert!(
        workflow.steps.len() >= 6,
        "Workflow should have at least 6 steps"
    );

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

    let final_prompt = &workflow.steps[5];
    assert_eq!(final_prompt.step_type, "prompt");
    assert_eq!(
        final_prompt.run_always,
        Some(true),
        "Final verdict prompt must run even after reviewer failure"
    );
    let final_prompt_content = final_prompt
        .content
        .as_ref()
        .expect("final prompt should include the reviewer execution result");
    assert!(final_prompt_content.contains("{{step_4_output}}"));
    assert!(final_prompt_content.contains("successful documenter does not"));

    let last = workflow.steps.last().expect("Workflow should have steps");
    assert_eq!(
        last.step_type, "action",
        "Last step should be the hard gate action"
    );
    assert_eq!(
        last.run_always,
        Some(true),
        "Hard gate must have run_always: true"
    );
    let last_cmd = last.cmd.as_ref().expect("Hard gate should have cmd");
    assert!(
        last_cmd.iter().any(|c| c.contains("STATE_OK")),
        "Hard gate must check STATE_OK"
    );
    assert!(
        last_cmd
            .iter()
            .any(|c| c.contains("REVIEW_FAIL_TASK_PENDING")),
        "Hard gate must allow a failed review to remain pending"
    );
}

#[test]
fn execute_task_implementation_prompt_matches_embedded_template() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);
    let workflow_path = tmp.path().join(".dec/workflows/execute_task.yaml");
    let project_content = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", workflow_path.display(), e));
    let project_workflow: Workflow =
        serde_yaml::from_str(&project_content).expect("project workflow should be valid YAML");
    let embedded_template: Workflow = serde_yaml::from_str(include_str!(
        "../src/project/templates/txt/workflow_execute_task.yaml"
    ))
    .expect("embedded workflow template should be valid YAML");

    assert_eq!(
        project_workflow.steps[2].content, embedded_template.steps[2].content,
        "The active project workflow implementation prompt must match the template"
    );
}

#[test]
fn execute_task_state_gate_accepts_only_completed_pass() {
    let tmp = TempDir::new().unwrap();
    let output = run_state_gate(&tmp, "PASS", "done", "x", "1 agent(s) completed");
    assert!(
        output.status.success(),
        "PASS gate failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("STATE_OK"));
    let progress = std::fs::read_to_string(tmp.path().join(".dec/state/progress.json")).unwrap();
    let tasks = std::fs::read_to_string(tmp.path().join("specs/tasks.md")).unwrap();
    assert!(progress.contains("\"status\":\"done\""));
    assert!(tasks.contains("- [x] [T057]"));
}

#[test]
fn execute_task_state_gate_fails_but_preserves_failed_task_as_pending() {
    let tmp = TempDir::new().unwrap();
    let output = run_state_gate(
        &tmp,
        "FAIL",
        "in_progress",
        " ",
        "Agent(s) failed: reviewer",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("REVIEW_FAIL_TASK_PENDING"));
    let progress = std::fs::read_to_string(tmp.path().join(".dec/state/progress.json")).unwrap();
    let tasks = std::fs::read_to_string(tmp.path().join("specs/tasks.md")).unwrap();
    let review =
        std::fs::read_to_string(tmp.path().join(".dec/agent-output/T057-review.md")).unwrap();
    assert!(progress.contains("\"status\":\"in_progress\""));
    assert!(tasks.contains("- [ ] [T057]"));
    assert!(review.contains("Fixture report retained"));
}

#[test]
fn documenter_instructions_keep_failed_review_incomplete() {
    let documenter = include_str!("../src/agent/builtins/documenter.yaml");
    assert!(documenter.contains("review-status.txt"));
    assert!(documenter.contains("For `FAIL`, set it to `in_progress`; never mark it `done`."));
    assert!(documenter
        .contains("For FAIL, leave or restore `[ ]`; the loop must not advance this task."));
    assert!(
        documenter.contains("Record the failed attempt and reason in `last_session.md` and memory")
    );
    assert!(documenter.contains("not list it among completed tasks"));
}
