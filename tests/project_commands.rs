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
fn test_project_init_level1() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["project", "init"], tmp.path());
    assert!(output.status.success());
    assert!(tmp.path().join(".dec").exists());
    assert!(tmp.path().join(".dec/config/project.toml").exists());
    assert!(tmp.path().join(".dec/isa/project.isa.md").exists());
    assert!(tmp.path().join(".dec/.gitignore").exists());
}

#[test]
fn test_project_init_standard() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["project", "init", "--standard"], tmp.path());
    assert!(output.status.success());
    assert!(tmp.path().join(".dec/workflows").exists());
    assert!(tmp
        .path()
        .join(".dec/workflows/implement_feature.yaml")
        .exists());
    assert!(tmp.path().join(".dec/state/progress.json").exists());
}

#[test]
fn test_project_init_full() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["project", "init", "--full"], tmp.path());
    assert!(output.status.success());
    assert!(tmp.path().join(".dec/isa/architecture.isa.md").exists());
    assert!(tmp.path().join(".dec/prompts/tasks").exists());
    assert!(tmp.path().join(".dec/knowledge/glossary.md").exists());
}

#[test]
fn test_project_init_aborts_if_exists() {
    let tmp = TempDir::new().unwrap();
    run_dectl(&["project", "init"], tmp.path());
    let output = run_dectl(&["project", "init"], tmp.path());
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"));
}

#[test]
fn test_project_info_with_dec() {
    let tmp = TempDir::new().unwrap();
    run_dectl(&["project", "init"], tmp.path());
    let output = run_dectl(&["project", "info", "--json"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("project-name"));
}

#[test]
fn test_project_info_without_dec() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["project", "info", "--json"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Missing .dec/config/project.toml"));
}

#[test]
fn test_project_scan() {
    let tmp = TempDir::new().unwrap();
    run_dectl(&["project", "init"], tmp.path());
    let output = run_dectl(&["project", "scan", "--depth", "3", "--json"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status"));
    assert!(stdout.contains("files"));
}

#[test]
fn test_project_scan_excludes_gitignored() {
    let tmp = TempDir::new().unwrap();
    run_dectl(&["project", "init"], tmp.path());
    fs::write(tmp.path().join("test.txt"), "test").unwrap();
    fs::create_dir(tmp.path().join("node_modules")).unwrap();
    fs::write(tmp.path().join("node_modules/package.json"), "{}").unwrap();
    let output = run_dectl(&["project", "scan", "--depth", "3", "--json"], tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("node_modules"));
}

#[test]
fn test_global_json_flag() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["project", "info", "--json"], tmp.path());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\":"));
}

#[test]
fn test_global_non_interactive_flag() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["--non-interactive", "project", "info"], tmp.path());
    assert!(output.status.success());
}

#[test]
fn test_project_context_text_output() {
    let tmp = TempDir::new().unwrap();
    run_dectl(&["project", "init", "--standard"], tmp.path());
    let output = run_dectl(&["project", "context", "--format", "text"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tokens:"));
    assert!(stdout.contains("/4000") || stdout.contains("/100"));
}

#[test]
fn test_project_context_json_output() {
    let tmp = TempDir::new().unwrap();
    run_dectl(&["project", "init", "--standard"], tmp.path());
    let output = run_dectl(&["project", "context", "--format", "json"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Valid JSON output");
    assert!(parsed.get("project").is_some());
    assert!(parsed.get("tokens_used").is_some());
    assert!(parsed.get("tokens_limit").is_some());
    assert!(parsed.get("files").is_some());
}

#[test]
fn test_project_context_max_tokens_truncate() {
    let tmp = TempDir::new().unwrap();
    run_dectl(&["project", "init", "--standard"], tmp.path());
    let output = run_dectl(
        &[
            "project",
            "context",
            "--format",
            "text",
            "--max-tokens",
            "100",
        ],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("/100"));
}

#[test]
fn test_project_context_no_dec_directory() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["project", "context"], tmp.path());
    assert!(!output.status.success());
    assert_eq!(output.status.code().unwrap(), 1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(".dec") || stderr.contains("not found"));
}

#[test]
fn test_project_init_type_api() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["project", "init", "--standard", "--type", "api"],
        tmp.path(),
    );
    assert!(output.status.success());
    assert!(tmp.path().join(".dec/workflows/test_api.yaml").exists());
    assert!(tmp
        .path()
        .join(".dec/workflows/document_endpoints.yaml")
        .exists());
    assert!(tmp.path().join(".dec/prompts/system/api.md").exists());
}

#[test]
fn test_project_init_type_cli() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["project", "init", "--standard", "--type", "cli"],
        tmp.path(),
    );
    assert!(output.status.success());
    assert!(tmp
        .path()
        .join(".dec/workflows/build_release.yaml")
        .exists());
    assert!(tmp
        .path()
        .join(".dec/workflows/document_args.yaml")
        .exists());
    assert!(tmp.path().join(".dec/prompts/system/cli.md").exists());
}

#[test]
fn test_project_init_type_microservice() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["project", "init", "--standard", "--type", "microservice"],
        tmp.path(),
    );
    assert!(output.status.success());
    assert!(tmp
        .path()
        .join(".dec/workflows/service_discovery.yaml")
        .exists());
    assert!(tmp.path().join(".dec/workflows/dockerize.yaml").exists());
    assert!(tmp
        .path()
        .join(".dec/workflows/inter_service_comm.yaml")
        .exists());
    assert!(tmp
        .path()
        .join(".dec/prompts/system/microservice.md")
        .exists());
}

#[test]
fn test_project_init_type_with_full() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["project", "init", "--full", "--type", "api"], tmp.path());
    assert!(output.status.success());
    assert!(tmp.path().join(".dec/workflows/test_api.yaml").exists());
    assert!(tmp.path().join(".dec/isa/architecture.isa.md").exists());
}

#[test]
fn test_project_init_type_other_no_extra_files() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["project", "init", "--standard", "--type", "other"],
        tmp.path(),
    );
    assert!(output.status.success());
    assert!(!tmp.path().join(".dec/prompts/system/api.md").exists());
    assert!(!tmp.path().join(".dec/prompts/system/cli.md").exists());
    assert!(!tmp
        .path()
        .join(".dec/prompts/system/microservice.md")
        .exists());
}

#[test]
fn test_project_init_invalid_type() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["project", "init", "--standard", "--type", "invalid"],
        tmp.path(),
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid project type"));
}
