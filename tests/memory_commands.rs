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

#[test]
fn test_memory_add() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["memory", "add", "Test memory content"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Added memory"));
}

#[test]
fn test_memory_add_with_tags() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["memory", "add", "Content with tags", "--tags", "rust,cli"],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rust"));
    assert!(stdout.contains("cli"));
}

#[test]
fn test_memory_list_empty() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["memory", "list", "--json"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status"));
}

#[test]
fn test_memory_list_with_json() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["memory", "add", "Entry one"], tmp.path());
    assert!(output.status.success());

    let output = run_dectl(&["memory", "list", "--json"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\":"));
    assert!(stdout.contains("entries"));
}

#[test]
fn test_memory_search() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["memory", "add", "Find me please"], tmp.path());
    assert!(output.status.success());

    let output = run_dectl(&["memory", "search", "Find me", "--json"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\":"));
    assert!(stdout.contains("Find me"));
}

#[test]
fn test_memory_search_no_results() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["memory", "search", "nonexistent query xyz", "--json"],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status"));
}

#[test]
fn test_memory_show_invalid_id() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["memory", "show", "99999"], tmp.path());
    assert!(!output.status.success());
}

#[test]
fn test_memory_add_and_show() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["memory", "add", "Show test content"], tmp.path());
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id_line = stdout
        .lines()
        .find(|l| l.contains("Added memory entry #"))
        .unwrap_or_default();
    let binding = id_line.replace("Added memory entry #", "");
    let id_str = binding.trim().to_string();

    let output = run_dectl(&["memory", "show", &id_str], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Show test content"));
}

#[test]
fn test_memory_global_json_flag() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["--json", "memory", "list"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\":"));
}

#[test]
fn test_memory_global_non_interactive() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["--non-interactive", "memory", "list"], tmp.path());
    assert!(output.status.success());
}
