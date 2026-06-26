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
    assert!(stdout.contains("\"content_preview\":"));
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
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Valid JSON");
    let id = parsed["id"].as_i64().expect("id field");

    let output = run_dectl(&["memory", "show", &id.to_string()], tmp.path());
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

#[test]
fn test_memory_add_with_type() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &[
            "memory",
            "add",
            "Test type decision",
            "--tags",
            "test",
            "--type",
            "decision",
        ],
        tmp.path(),
    );
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Valid JSON");
    let id = parsed["id"].as_i64().expect("id field");

    let output = run_dectl(&["memory", "show", &id.to_string()], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("decision"));
    assert!(stdout.contains("test"));
}

#[test]
fn test_memory_type_default() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["memory", "add", "Default type note"], tmp.path());
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Valid JSON");
    let id = parsed["id"].as_i64().expect("id field");

    let output = run_dectl(&["memory", "show", &id.to_string()], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"type\": \"note\""));
}

#[test]
fn test_memory_search_fts5() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["memory", "add", "FTS5 search test query"], tmp.path());
    assert!(output.status.success());

    let output = run_dectl(&["memory", "search", "FTS5 search", "--json"], tmp.path());
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FTS5 search test query"));
}

#[test]
fn test_memory_search_no_results_fts5() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(
        &["memory", "search", "xyznonexistent12345", "--json"],
        tmp.path(),
    );
    assert!(output.status.success());
}

#[test]
fn test_memory_query_type_filter() {
    let tmp = TempDir::new().unwrap();
    let proj = "type_filter_test";
    let output = run_dectl(
        &[
            "memory",
            "add",
            "Architecture decision",
            "--type",
            "decision",
            "--tags",
            "architecture",
            "--project",
            proj,
        ],
        tmp.path(),
    );
    assert!(output.status.success());

    let output = run_dectl(
        &["memory", "add", "General note", "--project", proj],
        tmp.path(),
    );
    assert!(output.status.success());

    let output = run_dectl(
        &[
            "memory",
            "query",
            &format!("type:decision AND project:{}", proj),
            "--json",
        ],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Architecture decision"));
    assert!(!stdout.contains("General note"));
}

#[test]
fn test_memory_query_tags_filter() {
    let tmp = TempDir::new().unwrap();
    let proj = "tags_filter_test";
    let output = run_dectl(
        &[
            "memory",
            "add",
            "Rust entry",
            "--tags",
            "rust,cli",
            "--project",
            proj,
        ],
        tmp.path(),
    );
    assert!(output.status.success());

    let output = run_dectl(
        &[
            "memory",
            "add",
            "Python entry",
            "--tags",
            "python",
            "--project",
            proj,
        ],
        tmp.path(),
    );
    assert!(output.status.success());

    let output = run_dectl(
        &[
            "memory",
            "query",
            &format!("tags:rust AND project:{}", proj),
            "--json",
        ],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rust entry"));
    assert!(!stdout.contains("Python entry"));
}

#[test]
fn test_memory_query_boolean_and() {
    let tmp = TempDir::new().unwrap();
    let proj = "bool_and_test";
    let output = run_dectl(
        &[
            "memory",
            "add",
            "Architecture design decision",
            "--type",
            "decision",
            "--tags",
            "architecture",
            "--project",
            proj,
        ],
        tmp.path(),
    );
    assert!(output.status.success());

    let output = run_dectl(
        &[
            "memory",
            "add",
            "Plain decision",
            "--type",
            "decision",
            "--tags",
            "general",
            "--project",
            proj,
        ],
        tmp.path(),
    );
    assert!(output.status.success());

    let output = run_dectl(
        &[
            "memory",
            "query",
            &format!("type:decision AND tags:architecture AND project:{}", proj),
            "--json",
        ],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Architecture design decision"));
    assert!(!stdout.contains("Plain decision"));
}

#[test]
fn test_memory_query_order_limit() {
    let tmp = TempDir::new().unwrap();
    let proj = "order_limit_test";
    let output = run_dectl(
        &["memory", "add", "First entry", "--project", proj],
        tmp.path(),
    );
    assert!(output.status.success());

    let output = run_dectl(
        &["memory", "add", "Second entry", "--project", proj],
        tmp.path(),
    );
    assert!(output.status.success());

    let output = run_dectl(
        &[
            "memory",
            "query",
            &format!("project:{} ORDER BY created DESC LIMIT 1", proj),
            "--json",
        ],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\"count\": 1"),
        "Should return exactly 1 result, got: {}",
        stdout
    );
}

#[test]
fn test_memory_query_invalid_syntax() {
    let tmp = TempDir::new().unwrap();
    let output = run_dectl(&["memory", "query", "invalid:value", "--json"], tmp.path());
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown field"));
}
