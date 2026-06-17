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

    // Phase 1: 4 diverse examples, no TaskFlow, traceability
    let examples = fs::read_to_string(tmp.path().join(".dec/sdd/references/examples.md")).unwrap();
    assert!(examples.contains("logsnap"), "examples.md missing logsnap");
    assert!(
        examples.contains("SnippetVault"),
        "examples.md missing SnippetVault"
    );
    assert!(
        examples.contains("LegacyPay"),
        "examples.md missing LegacyPay"
    );
    assert!(
        examples.contains("EventStream"),
        "examples.md missing EventStream"
    );
    assert!(
        examples.contains("HabitStack"),
        "examples.md missing HabitStack web example"
    );
    assert!(
        !examples.contains("TaskFlow"),
        "examples.md should not contain old TaskFlow example"
    );
    assert!(
        examples.contains("Traceability Matrix"),
        "examples.md missing Traceability Matrix"
    );

    // Phase 2: project-agnostic templates with new sections
    let templates =
        fs::read_to_string(tmp.path().join(".dec/sdd/references/templates.md")).unwrap();
    assert!(
        templates.contains("Program Type") || templates.contains("Interface Type"),
        "templates.md missing project-type-agnostic sections"
    );
    assert!(
        templates.contains("Edge Case Catalog"),
        "templates.md missing Edge Case Catalog"
    );
    assert!(
        templates.contains("Purity Boundaries"),
        "templates.md missing Purity Boundaries"
    );
    assert!(
        templates.contains("Drift Detection"),
        "templates.md missing Drift Detection"
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
        (260..=400).contains(&skill_lines),
        "SKILL.md has {} lines, expected 260-400",
        skill_lines
    );
    assert!(
        (480..=700).contains(&templates_lines),
        "templates.md has {} lines, expected 480-700",
        templates_lines
    );
    assert!(
        (1800..=2200).contains(&examples_lines),
        "examples.md has {} lines, expected 1800-2200",
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
