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
        "# Last Session\n\n**Date**: 2026-06-29\n",
    )
    .unwrap();
}

fn create_reqs_file(path: &Path) {
    fs::write(
        path,
        "# Biometric Auth\n\
         ## Description\n\
         Add fingerprint login to the app.\n\n\
         ### REQ-AUTH-001: Biometric Login\n\
         **User Story**:\n\
         > As a user, I want to log in with my fingerprint so that I don't need to type my password.\n\n\
         **Acceptance Criteria**:\n\
         - WHEN user has fingerprint enabled THEN the login screen SHALL show biometric option\n\
         - WHEN fingerprint matches THEN the system SHALL authenticate the user\n",
    )
    .unwrap();
}

fn create_specs_root(tmp: &TempDir) {
    fs::create_dir_all(tmp.path().join("specs")).unwrap();
    let spec_content = "# Master Spec\n\n## Functional Requirements\n\n### REQ-001: Existing Feature\n**User Story**:\n> As a user, I want an existing feature.\n\n---\n";
    let tasks_content =
        "# Tasks\n\n## Phase 1\n\n- [ ] [T001] [Setup] Initial setup — S (REQ-001)\n";
    fs::write(tmp.path().join("specs/spec.md"), spec_content).unwrap();
    fs::write(tmp.path().join("specs/tasks.md"), tasks_content).unwrap();
}

#[test]
fn test_spec_add_feature_from_file() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let reqs_path = tmp.path().join("req.md");
    create_reqs_file(&reqs_path);
    create_specs_root(&tmp);

    let output = run_dectl(
        &[
            "spec",
            "add",
            "biometric-auth",
            "--scope",
            "feature",
            "--from",
            reqs_path.to_str().unwrap(),
        ],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let spec_content = fs::read_to_string(tmp.path().join("specs/spec.md")).unwrap();
    assert!(
        spec_content.contains("REQ-002"),
        "Expected REQ-002 appended"
    );
    assert!(
        spec_content.contains("Biometric Login"),
        "Expected new REQ title"
    );

    let tasks_content = fs::read_to_string(tmp.path().join("specs/tasks.md")).unwrap();
    assert!(
        tasks_content.contains("[T002]"),
        "Expected T002 task appended"
    );
}

#[test]
fn test_spec_add_module_from_file() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let reqs_path = tmp.path().join("req.md");
    create_reqs_file(&reqs_path);
    create_specs_root(&tmp);

    let output = run_dectl(
        &[
            "spec",
            "add",
            "auth",
            "--scope",
            "module",
            "--from",
            reqs_path.to_str().unwrap(),
        ],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let module_dir = tmp.path().join("specs/auth");
    assert!(module_dir.exists(), "Module directory should exist");
    assert!(module_dir.join("constitution.md").exists());
    assert!(module_dir.join("spec.md").exists());
    assert!(module_dir.join("plan.md").exists());
    assert!(module_dir.join("tasks.md").exists());

    let mod_spec = fs::read_to_string(module_dir.join("spec.md")).unwrap();
    assert!(mod_spec.contains("REQ-AUTH-001"));
    assert!(mod_spec.contains("Biometric Login"));

    let root_spec = fs::read_to_string(tmp.path().join("specs/spec.md")).unwrap();
    assert!(
        root_spec.contains("[auth] Module"),
        "Root spec should reference module"
    );
}

#[test]
fn test_spec_add_no_specs_dir() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(&["spec", "add", "test", "--scope", "feature"], tmp.path());
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("specs/"),
        "Error should mention specs/ dir. Got: {}",
        stderr
    );
}

#[test]
fn test_spec_add_duplicate_module() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);
    create_specs_root(&tmp);

    fs::create_dir_all(tmp.path().join("specs/duplicate-mod")).unwrap();

    let output = run_dectl(
        &["spec", "add", "duplicate-mod", "--scope", "module"],
        tmp.path(),
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists") || stderr.contains("duplicate"),
        "Error should mention already exists. Got: {}",
        stderr
    );
}

#[test]
fn test_spec_add_module_non_interactive() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);
    create_specs_root(&tmp);

    let reqs_path = tmp.path().join("simple.md");
    fs::write(
        &reqs_path,
        "# Simple Module\nA basic module.\n\n### REQ-001: Core\n**User Story**:\n> As a user, I want core functionality.\n",
    )
    .unwrap();

    let output = run_dectl(
        &[
            "spec",
            "add",
            "simple-mod",
            "--scope",
            "module",
            "--from",
            reqs_path.to_str().unwrap(),
            "--non-interactive",
        ],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(tmp.path().join("specs/simple-mod/spec.md").exists());
}
