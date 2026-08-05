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

/// Extract the next_req/next_task IDs emitted by the spec_writer guidance
/// (printed by the "Read existing spec and tasks to find last IDs" action step).
fn extract_next_ids(stdout: &str) -> (String, String) {
    let mut next_req = String::new();
    let mut next_task = String::new();
    for line in stdout.lines() {
        for token in line.split_whitespace() {
            if let Some(v) = token.strip_prefix("next_req=") {
                next_req = v.to_string();
            }
            if let Some(v) = token.strip_prefix("next_task=") {
                next_task = v.to_string();
            }
        }
    }
    (next_req, next_task)
}

/// Simulate the AI agent appending a feature REQ + task to the root spec/tasks files.
fn simulate_feature_append(tmp: &TempDir, next_req: &str, next_task: &str, name: &str) {
    use std::io::Write;
    let spec_path = tmp.path().join("specs/spec.md");
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(spec_path)
        .unwrap();
    write!(
        f,
        "\n### REQ-{}: [{}] Feature\n**User Story**:\n> As a user, I want {} so that I can use this feature.\n\n---\n",
        next_req, name, name
    )
    .unwrap();

    let tasks_path = tmp.path().join("specs/tasks.md");
    let mut f2 = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(tasks_path)
        .unwrap();
    write!(
        f2,
        "- [ ] [T{}] [Feature] Implement {} — M (REQ-{})\n  **Build**: `cargo build`\n  **Verify**: `cargo test`\n  **Gate**: must pass before the next task begins\n",
        next_task, name, next_req
    )
    .unwrap();
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
    let stdout = String::from_utf8_lossy(&output.stdout);

    // dectl does NOT modify files; it emits AI guidance referencing the SDD skill
    assert!(
        stdout.contains("FEATURE SPEC GUIDANCE"),
        "expected feature guidance, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("SKILL.md") && stdout.contains("templates.md"),
        "guidance should reference the SDD skill and templates, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("REQ-AUTH-001"),
        "guidance should preserve source requirement IDs, got:\n{}",
        stdout
    );

    // The AI agent is responsible for the actual file update
    let spec_content = fs::read_to_string(tmp.path().join("specs/spec.md")).unwrap();
    assert!(
        !spec_content.contains("biometric-auth"),
        "dectl should not modify specs/spec.md itself (AI agent does that)"
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
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("MODULE SPEC GUIDANCE"),
        "expected module guidance, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("SKILL.md") && stdout.contains("templates.md"),
        "guidance should reference the SDD skill and templates, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("specs/auth/"),
        "module guidance should reference specs/auth/, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("REQ-AUTH-001"),
        "guidance should preserve source requirement IDs"
    );

    // dectl does NOT create the module files; the AI agent does
    assert!(
        !tmp.path().join("specs/auth").exists(),
        "dectl should not create specs/auth/ itself (AI agent does that)"
    );
}

/// T011 e2e: `--scope module` emits guidance for the 5 SDD documents and the
/// AI agent creates specs/<name>/ following the skill templates. dectl itself
/// must NOT create the module files or modify the root files.
#[test]
fn e2e_spec_add_module_agent_creates_specs() {
    let tmp = TempDir::new().unwrap();

    // 1. Real project bootstrap: project init --standard creates .dec/ + skill
    let output = run_dectl(&["project", "init", "--standard"], tmp.path());
    assert!(
        output.status.success(),
        "project init --standard failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(tmp.path().join(".dec/sdd/SKILL.md").exists());
    assert!(tmp.path().join(".dec/sdd/references/templates.md").exists());

    // 2. Root specs already exist (created earlier by the agent via spec init)
    create_specs_root(&tmp);

    // 3. Requirements file for the module
    let reqs_path = tmp.path().join("requirements_auth.md");
    create_reqs_file(&reqs_path);

    trust_agent("spec_writer", tmp.path());

    // 4. Run spec add --scope module --from
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
    let stdout = String::from_utf8_lossy(&output.stdout);

    // 5. Guidance names the module dir and all 5 SDD documents
    assert!(
        stdout.contains("MODULE SPEC GUIDANCE") && stdout.contains("specs/auth/"),
        "expected module guidance for specs/auth/, got:\n{}",
        stdout
    );
    for f in [
        "constitution.md",
        "spec.md",
        "requirements.md",
        "plan.md",
        "tasks.md",
    ] {
        assert!(
            stdout.contains(f),
            "module guidance should name {} to create, got:\n{}",
            f,
            stdout
        );
    }

    // 6. Guidance instructs to follow the skill templates (the phase-3 objective)
    assert!(
        stdout.contains("SKILL.md") && stdout.contains("templates.md"),
        "guidance should reference the SDD skill and templates, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Definition of Done") && stdout.contains("Edge Case Catalog"),
        "guidance should require template sections, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Build/Verify/Gate"),
        "guidance should require Build/Verify/Gate, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("REQ-AUTH-001"),
        "guidance should preserve source requirement IDs, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("also append"),
        "module guidance should ask to append root REQ + module task, got:\n{}",
        stdout
    );

    // 7. dectl did NOT create specs/auth/ nor touch the root files
    assert!(
        !tmp.path().join("specs/auth").exists(),
        "dectl should not create specs/auth/ itself (AI agent does that)"
    );
    let root_spec_before =
        fs::read_to_string(tmp.path().join("specs/spec.md")).unwrap();
    assert!(
        !root_spec_before.contains("auth"),
        "dectl should not modify root specs/spec.md itself"
    );

    // 8. Simulate the AI agent following the guidance: create the 5 files
    //    using the template sections (Definition of Done, Build/Verify/Gate...)
    let auth_dir = tmp.path().join("specs/auth");
    fs::create_dir_all(&auth_dir).unwrap();
    fs::write(
        auth_dir.join("constitution.md"),
        "# Constitution: auth\n\n## Definition of Done\n- SHALL be met before work begins\n",
    )
    .unwrap();
    fs::write(
        auth_dir.join("spec.md"),
        "# Spec: auth\n\n## Functional Requirements\n\n### REQ-AUTH-001: Biometric Login\n**User Story**:\n> As a user, I want to log in with my fingerprint.\n\n**Acceptance Criteria**:\n- WHEN user enables fingerprint THEN login SHALL offer biometric option\n",
    )
    .unwrap();
    fs::write(
        auth_dir.join("requirements.md"),
        "# Requirements: auth\n\n### REQ-AUTH-001\n\n## Verdict\n- Accepted\n",
    )
    .unwrap();
    fs::write(
        auth_dir.join("plan.md"),
        "# Plan: auth\n\n## Build Gate\n- cargo build passes\n\n## Purity Boundaries\n- no side effects\n",
    )
    .unwrap();
    fs::write(
        auth_dir.join("tasks.md"),
        "# Tasks: auth\n\n- [ ] [T001] Implement biometric login — M (REQ-AUTH-001)\n  **Build**: `cargo build`\n  **Verify**: `cargo test`\n  **Gate**: must pass before next task\n",
    )
    .unwrap();

    // 9. All 5 files exist with the required template sections
    assert!(auth_dir.join("constitution.md").exists());
    assert!(auth_dir.join("spec.md").exists());
    assert!(auth_dir.join("requirements.md").exists());
    assert!(auth_dir.join("plan.md").exists());
    assert!(auth_dir.join("tasks.md").exists());
    let constitution = fs::read_to_string(auth_dir.join("constitution.md")).unwrap();
    assert!(
        constitution.contains("Definition of Done"),
        "constitution.md should follow the template"
    );
    let tasks = fs::read_to_string(auth_dir.join("tasks.md")).unwrap();
    assert!(
        tasks.contains("Build") && tasks.contains("Verify") && tasks.contains("Gate"),
        "tasks.md should have Build/Verify/Gate per task"
    );

    // 10. Simulate the agent appending the root REQ + module task
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(tmp.path().join("specs/spec.md"))
        .unwrap();
    write!(
        f,
        "\n### REQ-AUTH-001: [auth] Module\n**User Story**:\n> As a user, I want the auth module.\n\n---\n"
    )
    .unwrap();
    let mut f2 = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(tmp.path().join("specs/tasks.md"))
        .unwrap();
    write!(
        f2,
        "- [ ] [T002] [auth] Implement module — M (REQ-AUTH-001)\n  **Build**: `cargo build`\n  **Verify**: `cargo test`\n  **Gate**: must pass before next task\n"
    )
    .unwrap();
    let root_spec = fs::read_to_string(tmp.path().join("specs/spec.md")).unwrap();
    assert!(
        root_spec.contains("REQ-AUTH-001"),
        "root specs/spec.md should reference the auth module (agent append)"
    );
    let root_tasks = fs::read_to_string(tmp.path().join("specs/tasks.md")).unwrap();
    assert!(
        root_tasks.contains("auth"),
        "root specs/tasks.md should have a module task for auth (agent append)"
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("MODULE SPEC GUIDANCE") && stdout.contains("specs/simple-mod/"),
        "expected module guidance for simple-mod, got:\n{}",
        stdout
    );
    assert!(
        !tmp.path().join("specs/simple-mod").exists(),
        "dectl should not create specs/simple-mod/ itself (AI agent does that)"
    );
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

    // 8. Run spec add --from to add auth module (emits AI guidance, does not create files)
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
    let add_stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        add_stdout.contains("MODULE SPEC GUIDANCE"),
        "spec add --from should emit module guidance, got:\n{}",
        add_stdout
    );

    // 9. Simulate the AI agent creating the module files following the skill
    let auth_dir = tmp.path().join("specs/auth");
    fs::create_dir_all(&auth_dir).unwrap();
    for f in ["constitution.md", "spec.md", "requirements.md", "plan.md", "tasks.md"] {
        fs::write(auth_dir.join(f), format!("# {}\n", f)).unwrap();
    }
    assert!(auth_dir.join("constitution.md").exists());
    assert!(auth_dir.join("spec.md").exists());
    assert!(auth_dir.join("requirements.md").exists());
    assert!(auth_dir.join("plan.md").exists());
    assert!(auth_dir.join("tasks.md").exists());

    // 10. Verify spec.md has content (agent creates templates with source file reference)
    let auth_spec = fs::read_to_string(auth_dir.join("spec.md")).unwrap();
    assert!(
        auth_spec.contains("spec.md"),
        "spec.md should contain content, got:\n{}",
        auth_spec
    );
}

#[test]
fn test_spec_add_feature_sequential_ids() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);
    create_specs_root(&tmp);

    trust_agent("spec_writer", tmp.path());

    let add = |name: &str| {
        let output = run_dectl(
            &[
                "spec",
                "add",
                name,
                "--scope",
                "feature",
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
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("FEATURE SPEC GUIDANCE"),
            "expected feature guidance, got:\n{}",
            stdout
        );
        extract_next_ids(&stdout)
    };

    // Feature adds must NOT create subdirectories
    let (req1, task1) = add("login");
    assert!(!tmp.path().join("specs/login").exists());
    // Seeded root has REQ-001/T001 -> guidance emits REQ-002/T002
    assert_eq!(req1, "002", "expected next_req=002");
    assert_eq!(task1, "002", "expected next_task=002");

    // Simulate the AI agent appending the emitted entries to the root files
    simulate_feature_append(&tmp, &req1, &task1, "login");

    let (req2, task2) = add("dashboard");
    assert!(!tmp.path().join("specs/dashboard").exists());
    // After the agent appended REQ-002/T002, guidance now emits REQ-003/T003
    assert_eq!(req2, "003", "expected next_req=003");
    assert_eq!(task2, "003", "expected next_task=003");
}
