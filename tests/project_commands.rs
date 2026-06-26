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

fn create_standard_dec(dec: &Path) {
    fs::create_dir_all(dec.join("isa")).unwrap();
    fs::create_dir_all(dec.join("config")).unwrap();
    fs::create_dir_all(dec.join("state")).unwrap();
    fs::create_dir_all(dec.join("prompts/system")).unwrap();
    fs::create_dir_all(dec.join("decisions")).unwrap();
    fs::write(
        dec.join("isa/project.isa.md"),
        "# My Project\nA test project",
    )
    .unwrap();
    fs::write(
        dec.join("config/project.toml"),
        "stack = \"Rust\"\nframework = \"Axum\"\n",
    )
    .unwrap();
    fs::write(
        dec.join("state/last_session.md"),
        "# Last Session\n\n**Fecha**: 2026-05-25\n\nDid some work.\n",
    )
    .unwrap();
    fs::write(
        dec.join("state/progress.json"),
        "{\"completed\": 5, \"total\": 10}",
    )
    .unwrap();
    fs::write(
        dec.join("prompts/system/integration.md"),
        "Integration prompt content",
    )
    .unwrap();
    fs::write(
        dec.join("decisions/001-db-choice.md"),
        "# Database choice\nWe chose SQLite.\n",
    )
    .unwrap();
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
fn test_context_proportional_budget() {
    let tmp = TempDir::new().unwrap();
    let dec = tmp.path().join(".dec");
    fs::create_dir_all(dec.join("isa")).unwrap();
    fs::create_dir_all(dec.join("config")).unwrap();
    fs::create_dir_all(dec.join("state")).unwrap();
    fs::create_dir_all(dec.join("prompts/system")).unwrap();
    fs::create_dir_all(dec.join("decisions")).unwrap();

    let big_content = "word pleasant orange grape banana apple ";
    let big_file: String = big_content.repeat(50);
    fs::write(dec.join("isa/project.isa.md"), &big_file).unwrap();
    fs::write(dec.join("config/project.toml"), &big_file).unwrap();
    fs::write(dec.join("state/last_session.md"), &big_file).unwrap();
    fs::write(dec.join("state/progress.json"), &big_file).unwrap();
    fs::write(dec.join("prompts/system/integration.md"), &big_file).unwrap();
    fs::write(dec.join("decisions/001-test.md"), &big_file).unwrap();

    let output = run_dectl(&["project", "context", "--max-tokens", "500"], tmp.path());
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sections_present = [
        "project.isa.md",
        "project.toml",
        "last_session.md",
        "progress.json",
        "integration.md",
        "001-test",
    ]
    .iter()
    .filter(|s| stdout.contains(*s))
    .count();
    assert!(
        sections_present >= 5,
        "Expected ≥5 sections, got {}:\n{}",
        sections_present,
        stdout
    );
}

#[test]
fn test_context_large_budget() {
    let tmp = TempDir::new().unwrap();
    let dec = tmp.path().join(".dec");
    fs::create_dir_all(dec.join("isa")).unwrap();
    let content = "hello world\n".repeat(50);
    fs::write(dec.join("isa/project.isa.md"), &content).unwrap();

    let output = run_dectl(&["project", "context", "--max-tokens", "10000"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tokens:"));
    assert!(stdout.contains("hello world"));
}

#[test]
fn test_context_extreme_budget() {
    let tmp = TempDir::new().unwrap();
    let dec = tmp.path().join(".dec");
    fs::create_dir_all(dec.join("isa")).unwrap();
    fs::write(dec.join("isa/project.isa.md"), "word ".repeat(500)).unwrap();

    let output = run_dectl(&["project", "context", "--max-tokens", "50"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "Output vacío con budget mínimo");
}

#[test]
fn test_context_format_compact() {
    let tmp = TempDir::new().unwrap();
    let dec = tmp.path().join(".dec");
    create_standard_dec(&dec);

    let output = run_dectl(&["project", "context", "--format", "compact"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() >= 4,
        "Expected ≥4 lines, got {}:\n{}",
        lines.len(),
        stdout
    );
    assert!(
        stdout.contains("\"project\":"),
        "Missing project: in:\n{}",
        stdout
    );
    assert!(
        stdout.contains("\"stack\":"),
        "Missing stack: in:\n{}",
        stdout
    );
}

#[test]
fn test_context_format_compact_json() {
    let tmp = TempDir::new().unwrap();
    let dec = tmp.path().join(".dec");
    create_standard_dec(&dec);

    let output = run_dectl(
        &["project", "context", "--format", "compact", "--json"],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Valid JSON");
    let data = parsed.get("data").expect("data field in envelope");
    assert!(data.get("project").is_some(), "Missing project field");
    assert!(data.get("stack").is_some(), "Missing stack field");
    assert!(data.get("progress").is_some(), "Missing progress field");
}

#[test]
fn test_context_recent_changes_prioritized() {
    let tmp = TempDir::new().unwrap();
    let dec = tmp.path().join(".dec");
    create_standard_dec(&dec);

    std::thread::sleep(std::time::Duration::from_millis(100));
    fs::write(
        dec.join("state/last_session.md"),
        "# Last Session\n\n**Fecha**: 2026-05-26\n\nJust modified this.\n",
    )
    .unwrap();

    let output = run_dectl(&["project", "context", "--max-tokens", "200"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Just modified this."));
}

#[test]
fn test_context_no_session_date() {
    let tmp = TempDir::new().unwrap();
    let dec = tmp.path().join(".dec");
    create_standard_dec(&dec);

    fs::remove_file(dec.join("state/last_session.md")).ok();

    let output = run_dectl(&["project", "context", "--max-tokens", "200"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
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
