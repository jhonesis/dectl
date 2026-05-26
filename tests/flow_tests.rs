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
        .expect("Failed to run dectl")
}

fn create_level2_dec(dec: &Path) {
    fs::create_dir_all(dec.join("isa")).unwrap();
    fs::create_dir_all(dec.join("config")).unwrap();
    fs::create_dir_all(dec.join("state")).unwrap();
    fs::create_dir_all(dec.join("prompts/system")).unwrap();
    fs::create_dir_all(dec.join("decisions")).unwrap();
    fs::write(
        dec.join("isa/project.isa.md"),
        "# My Test API\n\nA REST API for managing users.\nBuilt with Rust and Axum.",
    )
    .unwrap();
    fs::write(
        dec.join("config/project.toml"),
        "[project]\nname = \"test-api\"\ntype = \"api\"\n\n[stack]\nlanguages = [\"rust\"]\nframeworks = [\"axum\"]\ntools = []\ndatabases = []",
    )
    .unwrap();
    fs::write(
        dec.join("state/last_session.md"),
        "# Last Session\n\n**Fecha**: 2026-05-26\n**What was done**: Implemented user auth\n**What's pending**: Add refresh token\n**Próximo paso**: implementar refresh token\n",
    )
    .unwrap();
    fs::write(
        dec.join("state/progress.json"),
        r#"{"_comment":"Project status","features":[{"name":"user-auth","status":"done"},{"name":"refresh-token","status":"pending"}]}"#,
    )
    .unwrap();
    fs::write(
        dec.join("prompts/system/base.md"),
        "# System Prompt Base\n\nBasic instructions.",
    )
    .unwrap();
    fs::write(
        dec.join("prompts/system/integration.md"),
        "# Integration\n\nSession protocol instructions.",
    )
    .unwrap();
    fs::write(
        dec.join("decisions/001-auth-choice.md"),
        "# Decision: Auth\n\nJWT-based auth with refresh tokens.\n",
    )
    .unwrap();
}

#[test]
fn test_flow_new_project() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("new-project");
    fs::create_dir_all(&project_dir).unwrap();

    let output = run_dectl(&["project", "init", "--standard"], &project_dir);
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let dec = project_dir.join(".dec");
    assert!(
        dec.join("config/project.toml").exists(),
        "Missing project.toml"
    );
    assert!(dec.join("isa/project.isa.md").exists(), "Missing isa.md");
    assert!(
        dec.join("state/last_session.md").exists(),
        "Missing last_session.md"
    );
    assert!(
        dec.join("state/progress.json").exists(),
        "Missing progress.json"
    );
    assert!(dec.join("decisions").is_dir(), "Missing decisions dir");
    assert!(
        dec.join("prompts/system/base.md").exists(),
        "Missing base.md"
    );
    assert!(
        dec.join("prompts/system/integration.md").exists(),
        "Missing integration.md"
    );

    let context = run_dectl(&["project", "context", "--format", "text"], &project_dir);
    assert!(context.status.success(), "context failed");
    let ctx_stdout = String::from_utf8_lossy(&context.stdout);
    assert!(ctx_stdout.contains("tokens:"), "context missing token line");
    assert!(
        ctx_stdout.contains("project.toml") || ctx_stdout.contains("project.isa"),
        "context missing core files"
    );
}

#[test]
fn test_flow_session_resume() {
    let tmp = TempDir::new().unwrap();
    let dec = tmp.path().join(".dec");
    create_level2_dec(&dec);

    let output = run_dectl(&["project", "context", "--format", "text"], tmp.path());
    assert!(output.status.success(), "context failed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("refresh token"),
        "Context should contain 'Próximo paso' from last_session.md:\n{}",
        stdout
    );

    let json_output = run_dectl(&["project", "context", "--format", "json"], tmp.path());
    assert!(json_output.status.success());
    let json_stdout = String::from_utf8_lossy(&json_output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&json_stdout).expect("Valid JSON");
    assert!(parsed.get("project").is_some(), "Missing project field");
    assert!(parsed.get("tokens_used").is_some(), "Missing tokens_used");
    assert!(parsed.get("tokens_limit").is_some(), "Missing tokens_limit");
    assert!(parsed.get("files").is_some(), "Missing files array");
}

#[test]
fn test_flow_error_recovery() {
    let tmp = TempDir::new().unwrap();

    let no_args_output = run_dectl(&["memory", "add", "--json"], tmp.path());
    assert!(
        no_args_output.status.success() == false,
        "No-args add should fail"
    );
    assert_eq!(
        no_args_output.status.code().unwrap(),
        1,
        "Should exit with code 1"
    );
    let stderr = String::from_utf8_lossy(&no_args_output.stderr);
    assert!(
        stderr.contains("No content"),
        "Error should mention missing content, got: {}",
        stderr
    );

    let valid_output = run_dectl(
        &["memory", "add", "Implemented JWT auth system", "--json"],
        tmp.path(),
    );
    assert!(
        valid_output.status.success(),
        "Valid add should succeed, stderr: {}, stdout: {}",
        String::from_utf8_lossy(&valid_output.stderr),
        String::from_utf8_lossy(&valid_output.stdout)
    );
    let stdout = String::from_utf8_lossy(&valid_output.stdout);
    assert!(
        stdout.contains("\"status\": \"ok\""),
        "Should return ok status, got stdout: {}",
        stdout
    );
}

#[test]
fn test_flow_context_budget() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("budget-project");
    fs::create_dir_all(&project_dir).unwrap();

    let init = run_dectl(&["project", "init", "--standard"], &project_dir);
    assert!(init.status.success(), "init failed");

    let output = run_dectl(
        &[
            "project",
            "context",
            "--format",
            "text",
            "--max-tokens",
            "4000",
        ],
        &project_dir,
    );
    assert!(output.status.success(), "context failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "context output empty");
    assert!(
        stdout.contains("tokens:"),
        "context should show token budget"
    );
    assert!(
        stdout.contains("project.isa") || stdout.contains("project.toml"),
        "context should include core project files"
    );

    let json_output = run_dectl(
        &[
            "project",
            "context",
            "--format",
            "json",
            "--max-tokens",
            "4000",
            "--json",
        ],
        &project_dir,
    );
    assert!(json_output.status.success());
    let json_stdout = String::from_utf8_lossy(&json_output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&json_stdout).expect("Valid JSON");
    assert_eq!(parsed["status"], "ok", "JSON should be ok");
    let data = &parsed["data"];
    assert!(
        data.get("project").is_some(),
        "Missing project in JSON data"
    );
    assert!(data.get("tokens_used").is_some(), "Missing tokens_used");
    assert!(data.get("tokens_limit").is_some(), "Missing tokens_limit");
    assert!(data.get("files").is_some(), "Missing files array");
}
