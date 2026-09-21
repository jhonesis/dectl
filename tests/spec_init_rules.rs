//! Integration tests for T006 (REQ-rules-002, REQ-rules-005):
//! `spec init` questionnaire-once + `[PROFILE]` block + mirror
//! materialization + `active-ruleset.md`.
//!
//! The questionnaire itself (stdin parsing, `depends_on`, defaults) is
//! covered by unit tests in `src/rules/profile.rs` (injected I/O): the test
//! harness stdin is not a TTY, so e2e runs exercise the skip path and the
//! pre-written-`profile.toml` path (second-run semantics).

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

#[allow(clippy::too_many_arguments)]
fn write_profile(
    tmp: &TempDir,
    handles_pii: bool,
    has_sql: bool,
    public_api: bool,
    has_web_ui: bool,
    has_auth: bool,
    realtime: bool,
    cpu_intensive: bool,
    long_lived: bool,
) {
    let dir = tmp.path().join(".dec/rules");
    fs::create_dir_all(&dir).unwrap();
    let content = format!(
        "[profile]\nhandles_pii = {handles_pii}\nhas_sql = {has_sql}\npublic_api = {public_api}\n\
         has_web_ui = {has_web_ui}\nhas_auth = {has_auth}\nrealtime = {realtime}\n\
         cpu_intensive = {cpu_intensive}\nlong_lived = {long_lived}\n\n\
         [profile.meta]\nanswered_at = \"2026-09-18T00:00:00Z\"\n\
         answered_via = \"spec_init\"\nschema_version = 2\n"
    );
    fs::write(dir.join("profile.toml"), content).unwrap();
}

fn profile_all_false(tmp: &TempDir) {
    write_profile(tmp, false, false, false, false, false, false, false, false);
}

#[test]
fn spec_init_without_tty_skips_questionnaire_without_blocking() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    // No profile.toml + harness stdin without TTY → skip, success, pending.
    let output = run_dectl(&["spec", "init"], tmp.path());
    assert!(output.status.success());
    assert!(
        !tmp.path().join(".dec/rules/profile.toml").exists(),
        "skipped questionnaire must not create profile.toml"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("Rules pending") || stderr.contains("rules pending"),
        "must warn that rules are pending (stdout={stdout:?} stderr={stderr:?})"
    );
}

#[test]
fn spec_init_second_run_neither_reasks_nor_rematerializes() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);
    profile_all_false(&tmp);

    let first = run_dectl(&["spec", "init"], tmp.path());
    assert!(first.status.success());
    let stdout = String::from_utf8_lossy(&first.stdout);
    assert!(
        stdout.contains("[PROFILE]"),
        "message must include [PROFILE]"
    );
    assert!(
        stdout.to_lowercase().contains("re-pregunt") || stdout.to_lowercase().contains("re-ask"),
        "message must carry the no-re-ask instruction"
    );

    let profile_before = fs::read_to_string(tmp.path().join(".dec/rules/profile.toml")).unwrap();
    let transversal_before =
        fs::read_to_string(tmp.path().join(".dec/rules/transversal.yaml")).unwrap();
    let summary_before =
        fs::read_to_string(tmp.path().join(".dec/rules/active-ruleset.md")).unwrap();

    let second = run_dectl(&["spec", "init"], tmp.path());
    assert!(second.status.success());
    let stdout2 = String::from_utf8_lossy(&second.stdout);
    assert!(stdout2.contains("[PROFILE]"));

    assert_eq!(
        fs::read_to_string(tmp.path().join(".dec/rules/profile.toml")).unwrap(),
        profile_before,
        "second run must not re-ask (profile.toml unchanged)"
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join(".dec/rules/transversal.yaml")).unwrap(),
        transversal_before,
        "second run must not re-materialize (transversal unchanged)"
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join(".dec/rules/active-ruleset.md")).unwrap(),
        summary_before,
        "second run must not re-materialize (summary unchanged)"
    );
}

#[test]
fn spec_init_all_false_leaves_no_profiles_dir() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);
    profile_all_false(&tmp);

    let output = run_dectl(&["spec", "init"], tmp.path());
    assert!(output.status.success());
    assert!(
        !tmp.path().join(".dec/rules/profiles").exists(),
        "all-false must end with no profiles/"
    );
    assert!(tmp.path().join(".dec/rules/active-ruleset.md").is_file());
    assert!(tmp.path().join(".dec/rules/transversal.yaml").is_file());
}

#[test]
fn spec_init_has_auth_without_web_ui_omits_13_3() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);
    write_profile(&tmp, false, false, false, false, true, false, false, false);

    let output = run_dectl(&["spec", "init"], tmp.path());
    assert!(output.status.success());
    let path = tmp.path().join(".dec/rules/profiles/has_auth.yaml");
    assert!(path.is_file(), "has_auth flag must materialize its file");
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        !text.contains("13.3"),
        "mirror norm: inactive 13.3 must NOT be in has_auth.yaml"
    );
    let doc: serde_yaml::Value = serde_yaml::from_str(&text).unwrap();
    assert_eq!(doc.get("version").and_then(|v| v.as_u64()), Some(2));
}

#[test]
fn spec_init_profile_update_adds_13_3() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);
    write_profile(&tmp, false, false, false, false, true, false, false, false);

    let first = run_dectl(&["spec", "init"], tmp.path());
    assert!(first.status.success());
    let before = fs::read_to_string(tmp.path().join(".dec/rules/profiles/has_auth.yaml")).unwrap();
    assert!(!before.contains("13.3"));

    // Scope change: has_web_ui false → true regenerates everything.
    write_profile(&tmp, false, false, false, true, true, false, false, false);
    let second = run_dectl(&["spec", "init"], tmp.path());
    assert!(second.status.success());
    let after = fs::read_to_string(tmp.path().join(".dec/rules/profiles/has_auth.yaml")).unwrap();
    assert!(
        after.contains("13.3"),
        "enabling has_web_ui must add 13.3 to has_auth.yaml"
    );
    assert!(tmp
        .path()
        .join(".dec/rules/profiles/has_web_ui.yaml")
        .is_file());
}

#[test]
fn spec_init_realtime_without_public_api_renders_empty_rules() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);
    write_profile(&tmp, false, false, false, false, false, true, false, false);

    let output = run_dectl(&["spec", "init"], tmp.path());
    assert!(output.status.success());
    let path = tmp.path().join(".dec/rules/profiles/realtime.yaml");
    assert!(path.is_file());
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("rules: []"),
        "reference with no active rule must render `rules: []`, got:\n{text}"
    );
    let doc: serde_yaml::Value = serde_yaml::from_str(&text).unwrap();
    assert_eq!(doc.get("version").and_then(|v| v.as_u64()), Some(2));
}

#[test]
fn spec_init_materialized_files_carry_version_2() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);
    write_profile(&tmp, false, true, true, false, false, true, false, false);

    let output = run_dectl(&["spec", "init"], tmp.path());
    assert!(output.status.success());
    for rel in [
        ".dec/rules/transversal.yaml",
        ".dec/rules/profiles/has_sql.yaml",
        ".dec/rules/profiles/public_api.yaml",
        ".dec/rules/profiles/realtime.yaml",
        ".dec/rules/active-ruleset.md",
    ] {
        assert!(tmp.path().join(rel).is_file(), "{rel} must exist");
    }
    for rel in [
        ".dec/rules/transversal.yaml",
        ".dec/rules/profiles/has_sql.yaml",
        ".dec/rules/profiles/public_api.yaml",
        ".dec/rules/profiles/realtime.yaml",
    ] {
        let text = fs::read_to_string(tmp.path().join(rel)).unwrap();
        let doc: serde_yaml::Value = serde_yaml::from_str(&text).unwrap();
        assert_eq!(
            doc.get("version").and_then(|v| v.as_u64()),
            Some(2),
            "{rel} must carry version: 2"
        );
    }
    let md = fs::read_to_string(tmp.path().join(".dec/rules/active-ruleset.md")).unwrap();
    assert!(md.contains("schema_version 2"));
    let public_api =
        fs::read_to_string(tmp.path().join(".dec/rules/profiles/public_api.yaml")).unwrap();
    assert!(
        public_api.contains("11.4"),
        "public_api+realtime activates canonical 11.4"
    );
}
