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

    // Fused examples: TaskFlow (STANDARD) + LedgerPay (CRITICAL), no legacy examples
    let examples = fs::read_to_string(tmp.path().join(".dec/sdd/references/examples.md")).unwrap();
    assert!(examples.contains("TaskFlow"), "examples.md missing TaskFlow");
    assert!(
        examples.contains("LedgerPay"),
        "examples.md missing LedgerPay"
    );
    assert!(
        !examples.contains("logsnap"),
        "examples.md should not contain old logsnap example"
    );
    assert!(
        !examples.contains("SnippetVault"),
        "examples.md should not contain old SnippetVault example"
    );
    assert!(
        !examples.contains("LegacyPay"),
        "examples.md should not contain old LegacyPay example"
    );
    assert!(
        !examples.contains("EventStream"),
        "examples.md should not contain old EventStream example"
    );
    assert!(
        !examples.contains("HabitStack"),
        "examples.md should not contain old HabitStack example"
    );
    assert!(
        examples.contains("Build Gate"),
        "examples.md missing Build Gate in tasks"
    );

    // Fused templates: CRITICAL docs + BVG (Build/Verify/Gate) preserved
    let templates =
        fs::read_to_string(tmp.path().join(".dec/sdd/references/templates.md")).unwrap();
    assert!(
        templates.contains("[CRITICAL ONLY]"),
        "templates.md missing [CRITICAL ONLY] markers"
    );
    assert!(
        templates.contains("threat-model.md"),
        "templates.md missing threat-model.md template"
    );
    assert!(
        templates.contains("compliance-matrix.md"),
        "templates.md missing compliance-matrix.md template"
    );
    assert!(
        templates.contains("access-control-matrix.md"),
        "templates.md missing access-control-matrix.md template"
    );
    assert!(
        templates.contains("Task Readiness"),
        "templates.md missing Task Readiness checklist"
    );
    assert!(
        templates.contains("Constitution compliance review"),
        "templates.md missing Constitution compliance review"
    );

    // Phase 3: skill.md enhancements
    assert!(
        !skill.contains("quiero planificar"),
        "skill.md should have no Spanish"
    );
    assert!(skill.contains("Step 5"), "skill.md missing Step 5");
    assert!(
        skill.contains("dectl memory add"),
        "skill.md missing dectl memory integration"
    );
    assert!(
        skill.contains("Coordinator"),
        "skill.md missing Coordinator role"
    );
    assert!(
        skill.contains("Implementer"),
        "skill.md missing Implementer role"
    );
    assert!(skill.contains("Verifier"), "skill.md missing Verifier role");
    assert!(
        skill.contains("Clarification Phase"),
        "skill.md missing Clarification Phase"
    );
    assert!(
        skill.contains("WHAT vs HOW"),
        "skill.md missing WHAT vs HOW separation"
    );
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
fn test_spec_init_file_sizes() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(&["spec", "init"], tmp.path());
    assert!(output.status.success());

    let skill_lines = fs::read_to_string(tmp.path().join(".dec/sdd/SKILL.md"))
        .unwrap()
        .lines()
        .count();
    let templates_lines = fs::read_to_string(tmp.path().join(".dec/sdd/references/templates.md"))
        .unwrap()
        .lines()
        .count();
    let examples_lines = fs::read_to_string(tmp.path().join(".dec/sdd/references/examples.md"))
        .unwrap()
        .lines()
        .count();

    assert!(
        (480..=650).contains(&skill_lines),
        "SKILL.md has {} lines, expected 480-650",
        skill_lines
    );
    assert!(
        (800..=1000).contains(&templates_lines),
        "templates.md has {} lines, expected 800-1000",
        templates_lines
    );
    assert!(
        (750..=950).contains(&examples_lines),
        "examples.md has {} lines, expected 750-950",
        examples_lines
    );
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

#[test]
fn test_spec_init_from_passes_content_to_agent() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let reqs_path = tmp.path().join("requirements.md");
    fs::write(
        &reqs_path,
        "# Auth Requirements\n\nREQ-AUTH-001: User login\nREQ-AUTH-002: Password reset\n",
    )
    .unwrap();

    let output = run_dectl(&["spec", "init", "--from", "requirements.md"], tmp.path());
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[SOURCE FILE CONTENT]"),
        "Output missing [SOURCE FILE CONTENT] tag"
    );
    assert!(
        stdout.contains("[/SOURCE FILE CONTENT]"),
        "Output missing [/SOURCE FILE CONTENT] tag"
    );
    assert!(
        stdout.contains("REQ-AUTH-001: User login"),
        "Output missing file content"
    );
    assert!(
        stdout.contains("REQ-AUTH-002: Password reset"),
        "Output missing file content"
    );
    assert!(
        stdout.contains("Use the above content as input"),
        "Output missing agent instruction"
    );
}

#[test]
fn test_spec_init_from_nonexistent_file() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(&["spec", "init", "--from", "nonexistent.md"], tmp.path());
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("File not found"));
}

#[test]
fn test_spec_init_from_empty_file() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    fs::write(tmp.path().join("empty.md"), "").unwrap();

    let output = run_dectl(&["spec", "init", "--from", "empty.md"], tmp.path());
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("empty"));
}
