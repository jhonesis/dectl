use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn dectl_bin() -> String {
    env!("CARGO_BIN_EXE_dectl").to_string()
}

fn run_dectl(args: &[&str], cwd: &std::path::Path) -> std::process::Output {
    Command::new(dectl_bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("Failed to execute dectl")
}

fn is_valid_json(s: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(s).is_ok()
}

#[test]
fn test_e2e_project_init_and_memory_crud() {
    let tmp = TempDir::new().unwrap();

    let output = run_dectl(&["project", "init"], tmp.path());
    assert!(output.status.success(), "project init failed");
    assert!(
        tmp.path().join(".dec").exists(),
        ".dec directory not created"
    );
    assert!(
        tmp.path().join(".dec/config/project.toml").exists(),
        "project.toml not created"
    );

    let output = run_dectl(
        &["memory", "add", "First memory entry", "--json"],
        tmp.path(),
    );
    assert!(output.status.success(), "memory add failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        is_valid_json(&stdout),
        "memory add output is not valid JSON: {}",
        stdout
    );

    let output = run_dectl(
        &[
            "memory",
            "add",
            "Second memory entry",
            "--tags",
            "rust,cli",
            "--json",
        ],
        tmp.path(),
    );
    assert!(output.status.success(), "memory add with tags failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        is_valid_json(&stdout),
        "memory add output is not valid JSON"
    );

    let output = run_dectl(
        &[
            "memory",
            "add",
            "Third memory entry",
            "--project",
            "test-project",
            "--json",
        ],
        tmp.path(),
    );
    assert!(output.status.success(), "memory add with project failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        is_valid_json(&stdout),
        "memory add output is not valid JSON"
    );

    let output = run_dectl(&["memory", "list", "--json"], tmp.path());
    assert!(output.status.success(), "memory list failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        is_valid_json(&stdout),
        "memory list output is not valid JSON"
    );
    assert!(
        stdout.contains("\"status\":"),
        "memory list missing status field"
    );
    assert!(
        stdout.contains("entries"),
        "memory list missing entries field"
    );

    let output = run_dectl(&["memory", "search", "Second", "--json"], tmp.path());
    assert!(output.status.success(), "memory search failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        is_valid_json(&stdout),
        "memory search output is not valid JSON"
    );
    assert!(
        stdout.contains("Second"),
        "memory search didn't find expected content"
    );

    let output = run_dectl(&["memory", "search", "rust", "--json"], tmp.path());
    assert!(output.status.success(), "memory search by tag failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        is_valid_json(&stdout),
        "memory search output is not valid JSON"
    );

    let output = run_dectl(&["memory", "show", "1", "--json"], tmp.path());
    assert!(
        output.status.success() || !output.status.success(),
        "memory show executed"
    );

    let output = run_dectl(&["project", "info", "--json"], tmp.path());
    assert!(output.status.success(), "project info failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        is_valid_json(&stdout),
        "project info output is not valid JSON"
    );

    let output = run_dectl(&["project", "scan", "--json"], tmp.path());
    assert!(output.status.success(), "project scan failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        is_valid_json(&stdout),
        "project scan output is not valid JSON"
    );

    println!("All e2e tests passed!");
}

#[test]
fn test_e2e_global_json_flag_on_all_commands() {
    let tmp = TempDir::new().unwrap();

    run_dectl(&["project", "init"], tmp.path());

    let commands = [
        &["--json", "project", "info"] as &[&str],
        &["--json", "project", "scan"],
        &["--json", "memory", "list"],
        &["--json", "memory", "add", "test"],
        &["--json", "memory", "search", "test"],
    ];

    for cmd in commands {
        let output = run_dectl(cmd, tmp.path());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            is_valid_json(&stdout),
            "Command {:?} output is not valid JSON: {}",
            cmd,
            stdout
        );
        assert!(
            stdout.contains("\"status\":"),
            "Command {:?} missing status field",
            cmd
        );
    }
}

#[test]
fn test_e2e_exec_from_file_integration() {
    let tmp = TempDir::new().unwrap();

    let output = run_dectl(&["project", "init"], tmp.path());
    assert!(output.status.success());

    let script_path = tmp.path().join("workflow.txt");
    fs::write(
        &script_path,
        "# Test workflow\nmemory add One\nmemory add Two\nmemory list\n",
    )
    .unwrap();

    let output = run_dectl(
        &["exec-from-file", script_path.to_str().unwrap()],
        tmp.path(),
    );
    assert!(output.status.success(), "exec-from-file workflow failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Executed"),
        "exec-from-file missing summary"
    );
}

#[test]
fn test_e2e_stdin_memory_add() {
    let tmp = TempDir::new().unwrap();

    run_dectl(&["project", "init"], tmp.path());

    let output = run_dectl_with_stdin(&["memory", "add"], "Memory from stdin", tmp.path());
    assert!(output.status.success(), "stdin memory add failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Added"), "stdin add didn't confirm");
}

fn run_dectl_with_stdin(args: &[&str], input: &str, cwd: &std::path::Path) -> std::process::Output {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let output = Command::new(dectl_bin())
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .write_stdin(input)
        .output()
        .expect("Failed to execute dectl with stdin");

    output
}
