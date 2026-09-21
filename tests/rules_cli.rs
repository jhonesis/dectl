//! Integration tests for T007 @specs/rules/tasks.md (REQ-rules-003, REQ-rules-006):
//! the 6 `rules` subcommands (`list`, `search`, `resolve`, `context`,
//! `profile update`, `resync`) with stages, the 2000-token default budget
//! and `--json` on all of them.

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

fn stdout_text(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn write_profile_all(tmp: &TempDir, value: bool) {
    let dir = tmp.path().join(".dec/rules");
    fs::create_dir_all(&dir).unwrap();
    let v = if value { "true" } else { "false" };
    let content = format!(
        "[profile]\nhandles_pii = {v}\nhas_sql = {v}\npublic_api = {v}\n\
         has_web_ui = {v}\nhas_auth = {v}\nrealtime = {v}\n\
         cpu_intensive = {v}\nlong_lived = {v}\n\n\
         [profile.meta]\nanswered_at = \"2026-09-18T00:00:00Z\"\n\
         answered_via = \"spec_init\"\nschema_version = 2\n"
    );
    fs::write(dir.join("profile.toml"), content).unwrap();
}

fn json_data(out: &std::process::Output) -> serde_json::Value {
    let text = stdout_text(out);
    let envelope: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|_| panic!("not JSON: {text}"));
    assert_eq!(envelope["status"], "ok", "envelope must be ok: {text}");
    envelope["data"].clone()
}

#[test]
fn rules_help_documents_all_six_subcommands() {
    let tmp = TempDir::new().unwrap();
    let out = run_dectl(&["rules", "--help"], tmp.path());
    assert!(out.status.success());
    let text = stdout_text(&out);
    for sub in ["list", "search", "resolve", "context", "profile", "resync"] {
        assert!(text.contains(sub), "help must document `{sub}`:\n{text}");
    }
    let out = run_dectl(&["rules", "profile", "--help"], tmp.path());
    assert!(stdout_text(&out).contains("update"));
}

#[test]
fn list_browses_catalog_with_filters() {
    let tmp = TempDir::new().unwrap();
    let out = run_dectl(&["rules", "list"], tmp.path());
    assert!(out.status.success());
    assert!(stdout_text(&out).contains("# Rules (106)"));

    let out = run_dectl(&["rules", "list", "--severity", "must"], tmp.path());
    let text = stdout_text(&out);
    assert!(
        !text.contains("SHOULD"),
        "severity filter must apply:\n{text}"
    );

    let out = run_dectl(&["rules", "list", "--category", "Security"], tmp.path());
    let text = stdout_text(&out);
    assert!(text.contains("13.1"), "category filter must apply:\n{text}");
    assert!(
        !text.contains("Data Structures"),
        "other sections excluded:\n{text}"
    );

    let data = json_data(&run_dectl(&["--json", "rules", "list"], tmp.path()));
    assert_eq!(data["total"], 106);
}

#[test]
fn search_finds_sql_rule_case_insensitively() {
    let tmp = TempDir::new().unwrap();
    let out = run_dectl(&["rules", "search", "sql"], tmp.path());
    assert!(out.status.success());
    assert!(stdout_text(&out).contains("13.1"));
    let data = json_data(&run_dectl(
        &["--json", "rules", "search", "SQL"],
        tmp.path(),
    ));
    assert!(data["total"].as_u64().unwrap() >= 1);
}

#[test]
fn resolve_without_profile_gives_78_transversal() {
    let tmp = TempDir::new().unwrap();
    let data = json_data(&run_dectl(&["--json", "rules", "resolve"], tmp.path()));
    assert_eq!(data["active_total"], 78);
    assert_eq!(data["profile_present"], false);

    write_profile_all(&tmp, true);
    let data = json_data(&run_dectl(&["--json", "rules", "resolve"], tmp.path()));
    assert_eq!(data["active_total"], 106);
    assert_eq!(data["profile_present"], true);
}

#[test]
fn context_spec_returns_metadata_without_rule_text() {
    let tmp = TempDir::new().unwrap();
    let out = run_dectl(&["rules", "context", "--stage", "spec"], tmp.path());
    assert!(out.status.success());
    let text = stdout_text(&out);
    assert!(
        text.contains("handles_pii"),
        "metadata lists flags:\n{text}"
    );
    assert!(
        !text.contains("- **"),
        "spec stage must never carry rule text:\n{text}"
    );
}

#[test]
fn context_plan_truncates_with_omitted_footer() {
    let tmp = TempDir::new().unwrap();
    let data = json_data(&run_dectl(
        &[
            "--json",
            "rules",
            "context",
            "--stage",
            "plan",
            "--max-tokens",
            "100",
        ],
        tmp.path(),
    ));
    assert_eq!(data["stage"], "plan");
    assert_eq!(data["budget"], 100);
    assert!(
        data["omitted"].as_u64().unwrap() > 0,
        "tiny budget must omit: {data}"
    );
    assert!(
        data["context"]
            .as_str()
            .unwrap()
            .contains("rules omitted due to budget"),
        "footer required, never a silent cut: {data}"
    );
    // AVOID rules never reach the plan stage.
    let out = run_dectl(
        &[
            "rules",
            "context",
            "--stage",
            "plan",
            "--max-tokens",
            "100000",
        ],
        tmp.path(),
    );
    assert!(!stdout_text(&out).contains("AVOID"));
}

#[test]
fn context_review_is_blocking_plus_avoid() {
    let tmp = TempDir::new().unwrap();
    write_profile_all(&tmp, true);
    let out = run_dectl(
        &[
            "rules",
            "context",
            "--stage",
            "review",
            "--max-tokens",
            "100000",
        ],
        tmp.path(),
    );
    assert!(out.status.success());
    let text = stdout_text(&out);
    assert!(text.contains("13.1"), "blocking MUST present:\n{text}");
    assert!(
        !text.contains("14.5"),
        "design-advice MUST warns, never blocks:\n{text}"
    );
}

#[test]
fn context_task_accepts_newline_separated_files() {
    let tmp = TempDir::new().unwrap();
    write_profile_all(&tmp, true);
    // `git diff --name-only` style: one arg carrying newlines.
    let out = run_dectl(
        &[
            "rules",
            "context",
            "--stage",
            "task",
            "--files",
            "src/db.rs\nsrc/api.rs",
            "--task",
            "SQL injection fix",
            "--max-tokens",
            "100000",
        ],
        tmp.path(),
    );
    assert!(out.status.success());
    let text = stdout_text(&out);
    assert!(text.contains("src/db.rs"), "files echoed:\n{text}");
    let ids: Vec<&str> = text
        .lines()
        .filter_map(|l| l.strip_prefix("- **"))
        .filter_map(|rest| rest.split("**").next())
        .collect();
    let pos_13_1 = ids.iter().position(|id| *id == "13.1").unwrap();
    let pos_14_5 = ids.iter().position(|id| *id == "14.5").unwrap();
    assert!(pos_13_1 < pos_14_5, "relevant rule ranks first: {ids:?}");
}

#[test]
fn context_rejects_unknown_stage_by_name() {
    let tmp = TempDir::new().unwrap();
    let out = run_dectl(&["rules", "context", "--stage", "design"], tmp.path());
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(err.contains("design"), "error must name the stage: {err}");
}

#[test]
fn resync_without_profile_writes_transversal_and_schema_only() {
    let tmp = TempDir::new().unwrap();
    let out = run_dectl(&["rules", "resync"], tmp.path());
    assert!(out.status.success());
    let rules_dir = tmp.path().join(".dec/rules");
    assert!(rules_dir.join("transversal.yaml").is_file());
    assert!(rules_dir.join("profile.schema.yaml").is_file());
    assert!(
        !rules_dir.join("profiles").exists(),
        "no profile: profiles/ must not be created"
    );
    let data = json_data(&run_dectl(&["--json", "rules", "resync"], tmp.path()));
    assert_eq!(data["init_only"], true);
}

#[test]
fn resync_with_profile_regenerates_mirror() {
    let tmp = TempDir::new().unwrap();
    write_profile_all(&tmp, true);
    let out = run_dectl(&["rules", "resync"], tmp.path());
    assert!(out.status.success());
    assert!(tmp
        .path()
        .join(".dec/rules/profiles/has_sql.yaml")
        .is_file());
    assert!(tmp.path().join(".dec/rules/active-ruleset.md").is_file());
    let data = json_data(&run_dectl(&["--json", "rules", "resync"], tmp.path()));
    assert_eq!(data["active_total"], 106);
}

#[test]
fn profile_update_requires_interactive_terminal() {
    // Test harness stdin is not a TTY: must fail with guidance, never hang.
    let tmp = TempDir::new().unwrap();
    let out = run_dectl(&["rules", "profile", "update"], tmp.path());
    assert!(!out.status.success());
}

/// Spanish diacritics (plus inverted marks) that must never appear in
/// code-owned strings. Since T037 the embedded rule bodies are English too,
/// so this predicate guards headers, footers, metadata AND rule text.
fn has_spanish_diacritic(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(
            c,
            'á' | 'é'
                | 'í'
                | 'ó'
                | 'ú'
                | 'ü'
                | 'ñ'
                | 'Á'
                | 'É'
                | 'Í'
                | 'Ó'
                | 'Ú'
                | 'Ü'
                | 'Ñ'
                | '¿'
                | '¡'
        )
    })
}

#[test]
fn code_strings_carry_no_diacritics() {
    // Anti-regression (REQ-007, T035): headers, footers and spec-stage
    // metadata must stay diacritic-free.
    let tmp = TempDir::new().unwrap();
    // Spec stage carries metadata only, never rule text.
    let out = run_dectl(&["rules", "context", "--stage", "spec"], tmp.path());
    assert!(out.status.success());
    let text = stdout_text(&out);
    assert!(
        !has_spanish_diacritic(&text),
        "spec-stage context must carry no diacritics:\n{text}"
    );
    // Truncated plan output: only the footer line is code-owned.
    let out = run_dectl(
        &["rules", "context", "--stage", "plan", "--max-tokens", "100"],
        tmp.path(),
    );
    assert!(out.status.success());
    let text = stdout_text(&out);
    let footer = text
        .lines()
        .find(|l| l.contains("omitted due to budget"))
        .expect("truncated output must carry the omitted footer");
    assert!(
        !has_spanish_diacritic(footer),
        "footer must carry no diacritics: {footer}"
    );
    // Generated files: only the `#` header block is code-owned.
    let out = run_dectl(&["rules", "resync"], tmp.path());
    assert!(out.status.success());
    let transversal = fs::read_to_string(tmp.path().join(".dec/rules/transversal.yaml")).unwrap();
    let header: Vec<&str> = transversal
        .lines()
        .take_while(|l| l.starts_with('#'))
        .collect();
    assert!(
        header.iter().any(|l| l.contains("Generated by")),
        "header must be the English generated-by block:\n{transversal}"
    );
    for line in &header {
        assert!(
            !has_spanish_diacritic(line),
            "generated header must carry no diacritics: {line}"
        );
    }
    // Rule bodies (REQ-007, T037): the embedded catalog is fully English,
    // so task-stage context (condition → action lines) must be diacritic-free.
    let out = run_dectl(&["rules", "context", "--stage", "task"], tmp.path());
    assert!(out.status.success());
    let text = stdout_text(&out);
    assert!(
        text.contains("use an Array"),
        "task-stage context must carry the English catalog:\n{text}"
    );
    for line in text.lines().filter(|l| l.starts_with("- **")) {
        assert!(
            !has_spanish_diacritic(line),
            "rule body must carry no diacritics: {line}"
        );
    }
}
