use std::path::Path;
use std::{fs, process::Command};
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

fn create_legacy_project(dir: &Path) {
    fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "my-legacy-project"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["sqlite"] }
"#,
    )
    .unwrap();

    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("src/main.rs"),
        r#"use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
"#,
    )
    .unwrap();

    fs::write(
        dir.join("src/lib.rs"),
        r#"pub mod models;
pub mod handlers;
pub mod db;
"#,
    )
    .unwrap();

    fs::write(
        dir.join("README.md"),
        r#"# My Legacy Project

A REST API for managing user accounts.
Built with Rust, Axum, and SQLite.

## Endpoints
- GET /users - List users
- POST /users - Create user
- GET /users/:id - Get user
"#,
    )
    .unwrap();
}

#[test]
fn test_anchor_moment_full_flow() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("my-legacy-project");
    fs::create_dir_all(&project_dir).unwrap();
    create_legacy_project(&project_dir);

    let init_output = run_dectl(&["project", "init", "--standard"], &project_dir);
    assert!(
        init_output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init_output.stderr)
    );

    let dec = project_dir.join(".dec");
    assert!(dec.exists(), ".dec/ not created");

    let expected_core = [
        "config/project.toml",
        "isa/project.isa.md",
        "state/progress.json",
        "state/last_session.md",
    ];
    for rel_path in &expected_core {
        let full_path = dec.join(rel_path);
        assert!(full_path.exists(), "Missing: {}", rel_path);
        let content = fs::read_to_string(&full_path).unwrap_or_default();
        assert!(!content.trim().is_empty(), "Empty file: {}", rel_path);
    }

    let prompts_dir = dec.join("prompts/system");
    assert!(prompts_dir.exists(), "prompts/system directory missing");

    let agents_path = project_dir.join("AGENTS.md");
    assert!(agents_path.exists(), "Missing AGENTS.md");

    let project_toml = fs::read_to_string(dec.join("config/project.toml")).unwrap_or_default();
    assert!(
        project_toml.contains("rust"),
        "project.toml should contain detected stack:\n{}",
        project_toml
    );

    let isa_content = fs::read_to_string(dec.join("isa/project.isa.md")).unwrap_or_default();
    assert!(
        isa_content.len() > 50,
        "isa.md should have substantial content:\n{}",
        isa_content
    );

    let context_output = run_dectl(
        &[
            "project",
            "context",
            "--max-tokens",
            "4000",
            "--format",
            "text",
        ],
        &project_dir,
    );
    assert!(
        context_output.status.success(),
        "context failed: {}",
        String::from_utf8_lossy(&context_output.stderr)
    );
    let context_stdout = String::from_utf8_lossy(&context_output.stdout);
    assert!(!context_stdout.trim().is_empty(), "context output empty");
    assert!(
        context_stdout.contains("tokens:"),
        "context should show token count"
    );

    let compact_output = run_dectl(&["project", "context", "--format", "compact"], &project_dir);
    assert!(
        compact_output.status.success(),
        "compact failed: {}",
        String::from_utf8_lossy(&compact_output.stderr)
    );
    let compact_stdout = String::from_utf8_lossy(&compact_output.stdout);
    let compact_lines: Vec<&str> = compact_stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert!(
        compact_lines.len() <= 15,
        "compact should be ≤15 lines, got {}:\n{}",
        compact_lines.len(),
        compact_stdout
    );
    assert!(
        compact_stdout.contains("project:"),
        "compact missing project:"
    );
    assert!(compact_stdout.contains("stack:"), "compact missing stack:");
}

#[test]
fn test_anchor_moment_empty_project() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("empty-project");
    fs::create_dir_all(&project_dir).unwrap();

    let init_output = run_dectl(&["project", "init", "--standard"], &project_dir);
    assert!(
        init_output.status.success(),
        "init failed on empty project: {}",
        String::from_utf8_lossy(&init_output.stderr)
    );

    let dec = project_dir.join(".dec");
    assert!(dec.exists(), ".dec/ not created");

    let context_output = run_dectl(
        &["project", "context", "--max-tokens", "1000"],
        &project_dir,
    );
    assert!(context_output.status.success());
    let stdout = String::from_utf8_lossy(&context_output.stdout);
    assert!(!stdout.trim().is_empty());
}
