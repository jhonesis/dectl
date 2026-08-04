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

fn trust_agent(agent_name: &str, project_path: &std::path::Path) {
    let trust_file = dirs::home_dir()
        .unwrap_or_default()
        .join(".dectl")
        .join("trust.toml");
    std::fs::create_dir_all(trust_file.parent().unwrap()).ok();
    let canonical = std::fs::canonicalize(project_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| project_path.to_string_lossy().to_string());
    let entry = format!(
        "\n[[trusted]]\nproject_path = \"{}\"\nworkflow_name = \"{}\"\ntrusted_at = \"2026-08-03T00:00:00.000000+00:00\"\n",
        canonical, agent_name
    );
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trust_file)
        .expect("Failed to open trust.toml");
    f.write_all(entry.as_bytes())
        .expect("Failed to write trust entry");
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

    // Trust spec_writer agent (direct TOML write to avoid race condition)
    trust_agent("spec_writer", tmp.path());

    let output = run_dectl(
        &[
            "spec",
            "add",
            "biometric-auth",
            "--scope",
            "feature",
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

    // Verify root files were updated with feature content
    let spec_content = fs::read_to_string(tmp.path().join("specs/spec.md")).unwrap();
    assert!(
        spec_content.contains("REQ-") && spec_content.contains("biometric-auth"),
        "Root spec.md should contain REQ entry for biometric-auth"
    );
    let tasks_content = fs::read_to_string(tmp.path().join("specs/tasks.md")).unwrap();
    assert!(
        tasks_content.contains("[T") && tasks_content.contains("biometric-auth"),
        "Root tasks.md should contain task entry for biometric-auth"
    );
}

#[test]
fn test_spec_add_module_from_file() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let reqs_path = tmp.path().join("req.md");
    create_reqs_file(&reqs_path);
    create_specs_root(&tmp);

    // Trust spec_writer agent (direct TOML write to avoid race condition)
    trust_agent("spec_writer", tmp.path());

    let output = run_dectl(
        &[
            "spec",
            "add",
            "auth",
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

    let module_dir = tmp.path().join("specs/auth");
    assert!(module_dir.exists(), "Module directory should exist");
    assert!(module_dir.join("constitution.md").exists());
    assert!(module_dir.join("spec.md").exists());
    assert!(module_dir.join("plan.md").exists());
    assert!(module_dir.join("tasks.md").exists());
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

    // Trust spec_writer agent (direct TOML write to avoid race condition)
    trust_agent("spec_writer", tmp.path());

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

#[test]
fn e2e_spec_init_from_then_spec_add_from() {
    let tmp = TempDir::new().unwrap();

    // 1. Run project init --standard to create .dec/ structure
    let output = run_dectl(&["project", "init", "--standard"], tmp.path());
    assert!(
        output.status.success(),
        "project init --standard failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(tmp.path().join(".dec/sdd/SKILL.md").exists());
    assert!(tmp.path().join(".dec/config/project.toml").exists());

    // 2. Create requirements file for spec init
    let reqs1_path = tmp.path().join("project-reqs.md");
    fs::write(
        &reqs1_path,
        "# Mi Proyecto\n\
         Sistema de gestión de usuarios\n\n\
         ### REQ-001: Registro\n\
         **User Story**:\n\
         > As a user, I want to register\n\n\
         **Acceptance Criteria**:\n\
         - WHEN user submits form THEN system SHALL create account\n",
    )
    .unwrap();

    // 3. Run spec init --from (prints instructions for agent, doesn't create specs/)
    let output = run_dectl(
        &["spec", "init", "--from", reqs1_path.to_str().unwrap()],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "spec init --from failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // 4. Verify spec init --from output contains source file content
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[SOURCE FILE CONTENT]"),
        "spec init --from should output source file content"
    );
    assert!(
        stdout.contains("REQ-001"),
        "spec init --from should output requirement IDs"
    );

    // 5. Create specs/ directory manually (agent would do this in real workflow)
    fs::create_dir_all(tmp.path().join("specs")).unwrap();

    // 6. Trust spec_writer agent for this project (direct TOML write to avoid race condition)
    trust_agent("spec_writer", tmp.path());

    // 7. Create requirements file for spec add
    let reqs2_path = tmp.path().join("auth-reqs.md");
    fs::write(
        &reqs2_path,
        "# Auth Module\n\n\
         ### REQ-AUTH-001: Login\n\
         **User Story**:\n\
         > As a user, I want to login\n\n\
         **Acceptance Criteria**:\n\
         - WHEN user enters credentials THEN system SHALL authenticate\n\n\
         ### REQ-AUTH-002: Logout\n\
         **User Story**:\n\
         > As a user, I want to logout\n\n\
         **Acceptance Criteria**:\n\
         - WHEN user clicks logout THEN system SHALL clear session\n",
    )
    .unwrap();

    // 8. Run spec add --from to add auth module
    let output = run_dectl(
        &[
            "spec",
            "add",
            "auth",
            "--scope",
            "module",
            "--from",
            reqs2_path.to_str().unwrap(),
            "--non-interactive",
        ],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "spec add --from failed: stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // 9. Verify auth module was created
    let auth_dir = tmp.path().join("specs/auth");
    assert!(
        auth_dir.exists(),
        "specs/auth/ directory should exist after spec add"
    );
    assert!(auth_dir.join("constitution.md").exists());
    assert!(auth_dir.join("spec.md").exists());
    assert!(auth_dir.join("requirements.md").exists());
    assert!(auth_dir.join("plan.md").exists());
    assert!(auth_dir.join("tasks.md").exists());

    // 10. Verify spec.md has content (agent creates templates with source file reference)
    let auth_spec = fs::read_to_string(auth_dir.join("spec.md")).unwrap();
    assert!(
        auth_spec.contains("Specification"),
        "spec.md should contain specification header"
    );
    assert!(
        auth_spec.contains("auth") || auth_spec.contains("module"),
        "spec.md should reference the module name"
    );
}
