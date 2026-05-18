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
    assert!(stdout.contains("nombre-del-proyecto"));
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
