//! Integration tests for T008 @specs/rules/tasks.md (REQ-rules-007, REQ-rules-008):
//! the `execute_task` agent engagement — researcher writes
//! `{{task_id}}-rules.md`, coder includes it (absence marker, never aborts),
//! reviewer runs pre-triage + the §7.1 blocking set with `RULES_GATE` markers
//! and a machine gate (`exit 1` on FAIL), documenter records accepted
//! exceptions in `decisions/`.

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

/// Same as [`run_dectl`] but with the test binary's directory prepended to
/// PATH so nested `dectl` subprocess calls inside agent action steps resolve.
fn run_dectl_with_path(args: &[&str], cwd: &Path) -> std::process::Output {
    let bin_path = dectl_bin();
    let bin_dir = std::path::Path::new(&bin_path).parent().unwrap();
    let mut cmd = Command::new(&bin_path);
    cmd.args(args).current_dir(cwd);
    let path = std::env::var("PATH").unwrap_or_default();
    cmd.env("PATH", format!("{}:{path}", bin_dir.display()));
    cmd.output().expect("Failed to execute dectl")
}

fn combined(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Output of the final gate step only (the review prompt template itself
/// mentions every marker, so assertions must target the gate's own block).
fn gate_tail(text: &str) -> &str {
    match text.rfind("Step 14:") {
        Some(i) => &text[i..],
        None => text,
    }
}

fn init_standard(tmp: &TempDir) {
    let out = run_dectl(&["project", "init", "--standard"], tmp.path());
    assert!(
        out.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write_profile(tmp: &TempDir, has_sql: bool) {
    let dir = tmp.path().join(".dec/rules");
    fs::create_dir_all(&dir).unwrap();
    let bool_of = |v: bool| if v { "true" } else { "false" };
    let content = format!(
        "[profile]\nhandles_pii = false\nhas_sql = {}\npublic_api = false\n\
         has_web_ui = false\nhas_auth = false\nrealtime = false\n\
         cpu_intensive = false\nlong_lived = false\n\n\
         [profile.meta]\nanswered_at = \"2026-09-18T00:00:00Z\"\n\
         answered_via = \"spec_init\"\nschema_version = 2\n",
        bool_of(has_sql)
    );
    fs::write(dir.join("profile.toml"), content).unwrap();
}

fn describe_steps(agent: &str, cwd: &Path) -> Vec<serde_json::Value> {
    let out = run_dectl(&["--json", "agent", "describe", agent], cwd);
    assert!(out.status.success(), "describe {agent} failed");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let envelope: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|_| panic!("not JSON: {text}"));
    envelope["data"]["agent"]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("no steps array for {agent}: {text}"))
        .clone()
}

fn step_texts(steps: &[serde_json::Value]) -> Vec<String> {
    steps
        .iter()
        .map(|s| {
            let mut t = s["description"].as_str().unwrap_or("").to_string();
            t.push('\n');
            t.push_str(s["content"].as_str().unwrap_or(""));
            t.push('\n');
            if let Some(cmd) = s["cmd"].as_array() {
                for c in cmd {
                    t.push_str(c.as_str().unwrap_or(""));
                    t.push('\n');
                }
            }
            t
        })
        .collect()
}

fn assert_step_indices_valid(agent: &str, steps: &[serde_json::Value]) {
    let texts = step_texts(steps);
    let joined = texts.join("\n");
    let mut bad = Vec::new();
    // Every `step_N_output` reference must point at an existing 1-based step.
    let mut start = 0;
    while let Some(i) = joined[start..].find("step_") {
        let rest = &joined[start + i + 5..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if rest[digits.len()..].starts_with("_output") && !digits.is_empty() {
            let n: usize = digits.parse().unwrap();
            if n == 0 || n > steps.len() {
                bad.push(n);
            }
        }
        start += i + 5;
    }
    assert!(
        bad.is_empty(),
        "{agent}: dangling {{{{step_N_output}}}} refs ({} steps): {bad:?}",
        steps.len()
    );
}

#[test]
fn reviewer_step_indices_and_gate_wiring() {
    let tmp = TempDir::new().unwrap();
    let steps = describe_steps("reviewer", tmp.path());
    assert!(steps.len() >= 14, "reviewer must carry the rules steps");
    assert_step_indices_valid("reviewer", &steps);
    let texts = step_texts(&steps);

    assert!(
        texts.iter().any(|t| t.contains("pre-triage")),
        "reviewer must have a pre-triage step"
    );
    assert!(
        texts.iter().any(|t| t.contains("RULES_SUSPECT")),
        "pre-triage must emit RULES_SUSPECT markers"
    );
    assert!(
        texts.iter().any(|t| t.contains("§7.1")),
        "reviewer must reference the §7.1 blocking set"
    );
    assert!(
        texts.iter().any(|t| t.contains("RULES_GATE: FAIL")),
        "reviewer prompt must require the RULES_GATE marker"
    );
    assert!(
        texts.iter().any(|t| t.contains("RULES_GATE: SKIP")),
        "reviewer must declare SKIP without .dec/rules/"
    );

    let last = steps.last().expect("reviewer must have steps");
    assert_eq!(last["step_type"], "action", "gate must be the last step");
    assert_eq!(last["run_always"], true, "gate must have run_always: true");
    let last_cmd = last["cmd"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|c| c.as_str().unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    assert!(
        last_cmd.contains("RULES_GATE: FAIL") && last_cmd.contains("exit 1"),
        "gate must emit RULES_GATE: FAIL and exit 1"
    );
    assert!(
        last_cmd.contains("RULES_GATE: SKIP"),
        "gate must omit itself (SKIP) without .dec/rules/"
    );
}

#[test]
fn researcher_writes_task_rules_and_coder_includes_it() {
    let tmp = TempDir::new().unwrap();
    let steps = describe_steps("researcher", tmp.path());
    assert_step_indices_valid("researcher", &steps);
    assert!(
        step_texts(&steps)
            .iter()
            .any(|t| t.contains("rules context")),
        "researcher must fetch the task rules context"
    );
    assert!(
        step_texts(&steps).iter().any(|t| t.contains("-rules.md")),
        "researcher must write {{{{task_id}}}}-rules.md"
    );

    let steps = describe_steps("coder", tmp.path());
    assert_step_indices_valid("coder", &steps);
    let texts = step_texts(&steps);
    assert!(
        texts.iter().any(|t| t.contains("-rules.md")),
        "coder brief must include {{{{task_id}}}}-rules.md"
    );
    assert!(
        texts.iter().any(|t| t.contains("RULES_CONTEXT_MISSING")),
        "coder must continue with an absence marker (never abort)"
    );

    let steps = describe_steps("documenter", tmp.path());
    assert_step_indices_valid("documenter", &steps);
    assert!(
        step_texts(&steps).iter().any(|t| t.contains("decisions/")),
        "documenter must record accepted exceptions in decisions/"
    );
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_commit_base(tmp: &TempDir) {
    let run = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(tmp.path())
            .output()
            .expect("git failed");
        assert!(out.status.success(), "git {args:?}: {}", combined(&out));
    };
    run(&["init", "-q"]);
    run(&["config", "user.email", "t008@example.com"]);
    run(&["config", "user.name", "t008"]);
    run(&["config", "commit.gpgsign", "false"]);
    fs::write(tmp.path().join("app.rs"), "pub fn base() {}\n").unwrap();
    run(&["add", "."]);
    run(&["commit", "-qm", "base"]);
}

fn run_reviewer(tmp: &TempDir, task_id: &str) -> std::process::Output {
    run_dectl_with_path(
        &[
            "agent",
            "run",
            "reviewer",
            "--task",
            "t008 rules gate check",
            "--var",
            &format!("task_id={task_id}"),
            "--auto",
        ],
        tmp.path(),
    )
}

#[test]
fn gate_fails_on_concatenated_sql_with_has_sql() {
    if !git_available() {
        return;
    }
    let tmp = TempDir::new().unwrap();
    init_standard(&tmp);
    write_profile(&tmp, true);
    git_commit_base(&tmp);
    fs::write(
        tmp.path().join("app.rs"),
        "pub fn base() {}\npub fn find(id: &str) -> String {\n    let q = \"SELECT * FROM users WHERE id = \" + id;\n    q\n}\n",
    )
    .unwrap();

    let out = run_reviewer(&tmp, "T008-gate-fail");
    let text = combined(&out);
    assert!(
        text.contains("RULES_SUSPECT: 13.1"),
        "pre-triage must flag concatenated SQL:\n{text}"
    );
    let gate = gate_tail(&text);
    assert!(
        gate.contains("RULES_GATE: FAIL (ids: 13.1"),
        "gate must FAIL with has_sql active:\n{gate}"
    );
    assert!(
        !out.status.success(),
        "gate FAIL must break the pipeline (exit 1):\n{text}"
    );
}

#[test]
fn gate_passes_on_concatenated_sql_without_has_sql() {
    if !git_available() {
        return;
    }
    let tmp = TempDir::new().unwrap();
    init_standard(&tmp);
    write_profile(&tmp, false);
    git_commit_base(&tmp);
    fs::write(
        tmp.path().join("app.rs"),
        "pub fn base() {}\npub fn find(id: &str) -> String {\n    let q = \"SELECT * FROM users WHERE id = \" + id;\n    q\n}\n",
    )
    .unwrap();

    let out = run_reviewer(&tmp, "T008-gate-warn");
    let text = combined(&out);
    assert!(
        text.contains("RULES_SUSPECT: 13.1"),
        "pre-triage still suspects (recall), even inactive:\n{text}"
    );
    let gate = gate_tail(&text);
    assert!(
        gate.contains("RULES_GATE: PASS"),
        "gate must PASS when 13.1 is outside the active set:\n{gate}"
    );
    assert!(
        !gate.contains("RULES_GATE: FAIL"),
        "no FAIL outside the active set:\n{gate}"
    );
    assert!(out.status.success(), "pipeline continues:\n{text}");
}

#[test]
fn gate_passes_on_generic_diff_without_suspects() {
    if !git_available() {
        return;
    }
    let tmp = TempDir::new().unwrap();
    init_standard(&tmp);
    write_profile(&tmp, true);
    git_commit_base(&tmp);
    fs::write(
        tmp.path().join("app.rs"),
        "pub fn base() {}\n// TODO: rename this helper for clarity\npub fn helper() {}\n",
    )
    .unwrap();

    let out = run_reviewer(&tmp, "T008-gate-pass");
    let text = combined(&out);
    assert!(
        text.contains("RULES_SUSPECT: none"),
        "clean diff triages to none:\n{text}"
    );
    let gate = gate_tail(&text);
    assert!(
        gate.contains("RULES_GATE: PASS"),
        "generic advice warns, never fails:\n{gate}"
    );
    assert!(out.status.success(), "pipeline continues:\n{text}");
}

#[test]
fn gate_skips_without_dec_rules() {
    if !git_available() {
        return;
    }
    let tmp = TempDir::new().unwrap();
    init_standard(&tmp);
    fs::remove_dir_all(tmp.path().join(".dec/rules")).unwrap();
    git_commit_base(&tmp);
    fs::write(
        tmp.path().join("app.rs"),
        "pub fn base() {}\npub fn find(id: &str) -> String {\n    let q = \"SELECT * FROM users WHERE id = \" + id;\n    q\n}\n",
    )
    .unwrap();

    let out = run_reviewer(&tmp, "T008-gate-skip");
    let text = combined(&out);
    let gate = gate_tail(&text);
    assert!(
        gate.contains("RULES_GATE: SKIP"),
        "gate is omitted without .dec/rules/:\n{gate}"
    );
    assert!(
        !gate.contains("RULES_GATE: FAIL"),
        "SKIP never fails:\n{gate}"
    );
    assert!(out.status.success(), "pipeline continues:\n{text}");
}

#[test]
fn researcher_run_writes_task_rules_file() {
    let tmp = TempDir::new().unwrap();
    init_standard(&tmp);
    let out = run_dectl_with_path(
        &[
            "agent",
            "run",
            "researcher",
            "--task",
            "t008 rules file check",
            "--var",
            "task_id=T008-rules-file",
            "--auto",
        ],
        tmp.path(),
    );
    let text = combined(&out);
    assert!(out.status.success(), "researcher must succeed:\n{text}");
    let rules_file = tmp
        .path()
        .join(".dec/agent-output/T008-rules-file-rules.md");
    assert!(
        rules_file.is_file(),
        "researcher must write {{{{task_id}}}}-rules.md:\n{text}"
    );
    let body = fs::read_to_string(&rules_file).unwrap();
    assert!(
        body.contains("Rules context") || body.contains("RULES_CONTEXT"),
        "rules file carries the stage-task context:\n{body}"
    );
}

#[test]
fn coder_run_marks_missing_rules_context_without_aborting() {
    let tmp = TempDir::new().unwrap();
    init_standard(&tmp);
    let out = run_dectl_with_path(
        &[
            "agent",
            "run",
            "coder",
            "--task",
            "t008 coder absence check",
            "--var",
            "task_id=T008-coder-absence",
            "--auto",
        ],
        tmp.path(),
    );
    let text = combined(&out);
    assert!(
        out.status.success(),
        "coder must not abort without rules context:\n{text}"
    );
    let ctx = tmp
        .path()
        .join(".dec/agent-output/T008-coder-absence-coder-context.md");
    assert!(ctx.is_file(), "coder context must be written:\n{text}");
    let body = fs::read_to_string(&ctx).unwrap();
    assert!(
        body.contains("RULES_CONTEXT_MISSING"),
        "coder context carries the absence marker:\n{body}"
    );
}
