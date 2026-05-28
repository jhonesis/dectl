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

fn create_dec_base(tmp: &TempDir) {
    fs::create_dir_all(tmp.path().join(".dec/config")).unwrap();
    fs::create_dir_all(tmp.path().join(".dec/isa")).unwrap();
    fs::create_dir_all(tmp.path().join(".dec/state")).unwrap();
    fs::write(
        tmp.path().join(".dec/config/project.toml"),
        "[project]\nname = \"test-project\"\nproject_type = \"other\"\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".dec/isa/project.isa.md"),
        "# Project ISA\n\n## Identity\nName: test-project\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".dec/state/last_session.md"),
        "# Last Session\n\n**Date**: 2026-05-28\n",
    )
    .unwrap();
}

#[test]
fn test_spec_init_creates_sdd_dir() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(&["spec", "init"], tmp.path());
    assert!(output.status.success());

    assert!(tmp.path().join(".dec/sdd/SKILL.md").exists());
    assert!(tmp.path().join(".dec/sdd/references/templates.md").exists());
    assert!(tmp.path().join(".dec/sdd/references/examples.md").exists());

    let skill = fs::read_to_string(tmp.path().join(".dec/sdd/SKILL.md")).unwrap();
    assert!(skill.contains("Spec-Driven Development"));
    assert!(skill.contains("Build: + Verify: + Gate:"));
}

#[test]
fn test_spec_init_bridge_updates() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(&["spec", "init"], tmp.path());
    assert!(output.status.success());

    let toml = fs::read_to_string(tmp.path().join(".dec/config/project.toml")).unwrap();
    assert!(toml.contains("[specs]"));
    assert!(toml.contains("dir = \"specs\""));

    let isa = fs::read_to_string(tmp.path().join(".dec/isa/project.isa.md")).unwrap();
    assert!(isa.contains("See `specs/` for SDD artifacts"));
}

#[test]
fn test_spec_init_no_dec_error() {
    let tmp = TempDir::new().unwrap();

    let output = run_dectl(&["spec", "init"], tmp.path());
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(".dec/ not found"));
}

#[test]
fn test_spec_init_json_output() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(&["spec", "init", "--json"], tmp.path());
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\": \"ok\""));
    assert!(stdout.contains(".dec/sdd/ ready"));
}

#[test]
fn test_spec_init_idempotent() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let first = run_dectl(&["spec", "init"], tmp.path());
    assert!(first.status.success());

    let second = run_dectl(&["spec", "init"], tmp.path());
    assert!(second.status.success());
}

#[test]
fn test_spec_init_standard_includes_sdd() {
    let tmp = TempDir::new().unwrap();

    let output = run_dectl(&["project", "init", "--standard"], tmp.path());
    assert!(output.status.success());

    assert!(tmp.path().join(".dec/sdd/SKILL.md").exists());
    assert!(tmp.path().join(".dec/sdd/references/templates.md").exists());
    assert!(tmp.path().join(".dec/sdd/references/examples.md").exists());
}
