//! Continuous-improvement loop (REQ-012 / T063).
//!
//! Checks the three pieces end to end:
//! 1. `dectl spec feedback` prints the reusable pass/fail evaluation template
//!    and the per-project memory contract;
//! 2. feedback is recorded in project memory with the declared tags;
//! 3. a non-fit item recurring in two independent projects is emitted as
//!    evidence for the emitting system's proposal pipeline, with no local
//!    changelog.

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

fn create_project(tmp: &TempDir, name: &str) {
    fs::create_dir_all(tmp.path().join(".dec/config")).unwrap();
    fs::write(
        tmp.path().join(".dec/config/project.toml"),
        format!(
            "[project]\nname = \"{name}\"\nproject_type = \"other\"\ndescription = \"fixture\"\n\n\
             [stack]\nlanguages = [\"rust\"]\n"
        ),
    )
    .unwrap();
}

/// Globally unique per run: feedback records persist in the shared database
/// across runs, so a bare PID could eventually collide with old records.
fn unique(suffix: &str) -> String {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("t063-{suffix}-{}-{stamp}", std::process::id())
}

/// Project names are unique per test so recurrence never crosses tests.
fn project_name(test: &str, side: &str) -> String {
    format!("{}-{}-{side}", unique(test), std::process::id())
}

#[test]
fn test_spec_feedback_prints_reusable_evaluation_template() {
    let tmp = TempDir::new().unwrap();
    create_project(&tmp, &project_name("template", "a"));

    let output = run_dectl(&["spec", "feedback"], tmp.path());
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("[GENERATOR FEEDBACK LOOP]"));
    assert!(stdout.contains("dectl memory add"));
    assert!(stdout.contains("generator-defect"));
    assert!(stdout.contains("generator-non-fit"));
    assert!(stdout.contains("generator-decision"));

    assert!(stdout.contains("Generator Evaluation — pass/fail template"));
    assert!(stdout.contains("| compiles | inside |"));
    assert!(stdout.contains("| starts | inside |"));
    assert!(stdout.contains("| business_rule | outside |"));
    assert!(stdout.contains("evaluation FAILS"));
    assert!(stdout.contains("Reuse this template unchanged"));
}

#[test]
fn test_non_fit_recurring_in_two_projects_is_fed_as_evidence() {
    let first = TempDir::new().unwrap();
    let second = TempDir::new().unwrap();
    let first_name = project_name("recurring", "a");
    let second_name = project_name("recurring", "b");
    create_project(&first, &first_name);
    create_project(&second, &second_name);
    let id = unique("NF-1");
    let origin = "§2.1";
    let reason = "rule does not map to a declarative entry";

    // First project alone records the item.
    let output = run_dectl(
        &[
            "spec",
            "feedback",
            "--kind",
            "non-fit",
            "--id",
            &id,
            "--origin",
            origin,
            "--content",
            reason,
        ],
        first.path(),
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // One project alone is not recurrence.
    let output = run_dectl(&["spec", "feedback", "--report"], first.path());
    assert!(output.status.success());
    let single = String::from_utf8_lossy(&output.stdout);
    assert!(single.contains("[NON-FIT PROPOSAL EVIDENCE]"));
    assert!(
        !single.contains(&id),
        "a single project must not count as recurrence:\n{single}"
    );

    // A second independent project records the same stable id.
    let output = run_dectl(
        &[
            "spec",
            "feedback",
            "--kind",
            "non-fit",
            "--id",
            &id,
            "--origin",
            origin,
            "--content",
            reason,
        ],
        second.path(),
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Two independent projects are.
    let output = run_dectl(&["spec", "feedback", "--report"], second.path());
    assert!(output.status.success());
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(
        report.contains(&id),
        "recurring item must be reported:\n{report}"
    );
    assert!(report.contains(&first_name) && report.contains(&second_name));
    assert!(report.contains("geneia-cli"));
    assert!(report.contains("plangenecli.md A2.3"));
    assert!(report.contains("dectl keeps no local changelog"));
}

#[test]
fn test_feedback_tags_are_recorded_per_project() {
    let tmp = TempDir::new().unwrap();
    let name = project_name("tags", "a");
    create_project(&tmp, &name);

    let defect = run_dectl(
        &[
            "spec",
            "feedback",
            "--kind",
            "defect",
            "--content",
            "generator dropped a rule",
        ],
        tmp.path(),
    );
    assert!(defect.status.success());
    let stdout = String::from_utf8_lossy(&defect.stdout);
    assert!(stdout.contains("generator-defect"));
    assert!(stdout.contains(&name));

    let decision = run_dectl(
        &[
            "spec",
            "feedback",
            "--kind",
            "decision",
            "--content",
            "keep the complete suite as default",
        ],
        tmp.path(),
    );
    assert!(decision.status.success());
    let stdout = String::from_utf8_lossy(&decision.stdout);
    assert!(stdout.contains("generator-decision"));
    assert!(stdout.contains(&name));
}

#[test]
fn test_non_fit_requires_stable_id_and_origin() {
    let tmp = TempDir::new().unwrap();
    create_project(&tmp, &project_name("invalid", "a"));

    let output = run_dectl(
        &[
            "spec",
            "feedback",
            "--kind",
            "non-fit",
            "--content",
            "missing id and origin",
        ],
        tmp.path(),
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("stable id and an origin"));
}

#[test]
fn test_project_init_writes_the_reusable_evaluation_template() {
    let tmp = TempDir::new().unwrap();

    let output = run_dectl(&["project", "init", "--standard"], tmp.path());
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let template_path = tmp.path().join(".dec/sdd/generator-evaluation.md");
    assert!(
        template_path.exists(),
        "project init must write the evaluation template"
    );
    let template = fs::read_to_string(&template_path).unwrap();
    assert!(template.contains("Generator Evaluation — pass/fail template"));
    assert!(template.contains("| compiles | inside |"));
    assert!(template.contains("| starts | inside |"));
    assert!(template.contains("| business_rule | outside |"));
}
