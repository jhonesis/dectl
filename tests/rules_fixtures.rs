//! Fixtures + reference-output regression tests (T004, REQ-rules-005).
//!
//! - `tests/fixtures/rules/` holds the 4 format samples copied byte-identical
//!   from `rules/`. Each documents a different producing command: `transversal`
//!   samples `project init --standard` output; the other three sample what
//!   `spec init` materializes for those flags (never `project init`, which
//!   never touches `profiles/`).
//! - `tests/fixtures/rules/reference/` holds a committed copy of the full
//!   reference output of `rules/partition_rules.py` (= materialization with
//!   an all-true profile). It lives in-repo (not in `../rules/profiles`,
//!   which only exists in the outer big-project layout) so CI sees it.
//!   Regenerate with the script and copy over when the catalog changes.
//!   These tests validate that reference the same way `--check` does:
//!   78 + 3 + 6 + 7 + 11 + 1 = 106 canonical rules, 0 duplicated IDs, and
//!   `realtime` as reference-only (`11.4` lives in `public_api.yaml`).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `dectl/tests/fixtures/rules/` — the 4 format samples.
fn fixtures_dir() -> PathBuf {
    crate_dir().join("tests").join("fixtures").join("rules")
}

/// `tests/fixtures/rules/reference/` — committed copy of the full
/// partition output (in-repo so CI and fresh clones see it).
fn reference_profiles_dir() -> PathBuf {
    fixtures_dir().join("reference")
}

fn parse_yaml_docs(path: &std::path::Path) -> Vec<serde_yaml::Value> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("fixture must be readable: {}", path.display()));
    // `public_api_and_realtime.yaml` holds two documents (canonical
    // `public_api` + reference-only `realtime`) separated by `---`.
    text.split("\n---\n")
        .map(|doc| {
            serde_yaml::from_str(doc.trim())
                .unwrap_or_else(|e| panic!("fixture must parse as YAML ({}): {e}", path.display()))
        })
        .collect()
}

fn parse_yaml(path: &std::path::Path) -> serde_yaml::Value {
    let docs = parse_yaml_docs(path);
    assert_eq!(
        docs.len(),
        1,
        "{} must hold a single YAML document",
        path.display()
    );
    docs.into_iter().next().unwrap()
}

fn rules_sequence(doc: &serde_yaml::Value, path: &std::path::Path) -> Vec<serde_yaml::Value> {
    doc.get("rules")
        .and_then(|v| v.as_sequence())
        .unwrap_or_else(|| panic!("{} must have a `rules:` sequence", path.display()))
        .clone()
}

#[test]
fn fixtures_exist_and_carry_schema_version_2() {
    for name in [
        "transversal.yaml",
        "has_sql.yaml",
        "has_auth.yaml",
        "public_api_and_realtime.yaml",
    ] {
        let path = fixtures_dir().join(name);
        assert!(path.is_file(), "missing fixture: {}", path.display());
        for doc in parse_yaml_docs(&path) {
            assert_eq!(
                doc.get("version").and_then(|v| v.as_u64()),
                Some(2),
                "{name} must carry `version: 2` (current schema_version)"
            );
        }
    }
}

#[test]
fn fixtures_document_distinct_producing_commands() {
    // `transversal.yaml` samples `project init --standard` output: universal
    // rules only, 4 of the 78 (format sample, not exhaustive).
    let transversal = parse_yaml(&fixtures_dir().join("transversal.yaml"));
    assert_eq!(
        transversal.get("source").and_then(|v| v.as_str()),
        Some("development-rules.md")
    );
    let rules = rules_sequence(&transversal, &fixtures_dir().join("transversal.yaml"));
    assert_eq!(rules.len(), 4, "transversal fixture is a 4-rule sample");
    assert!(
        rules.iter().all(|r| r
            .get("requires")
            .and_then(|v| v.as_sequence())
            .map(|s| s.is_empty())
            .unwrap_or(false)),
        "sampled transversal rules must have empty `requires`"
    );

    // `has_sql.yaml` samples `spec init` materialization for a simple flag.
    let has_sql = parse_yaml(&fixtures_dir().join("has_sql.yaml"));
    assert_eq!(
        has_sql.get("profile_flag").and_then(|v| v.as_str()),
        Some("has_sql")
    );
    let ids: Vec<&str> = has_sql
        .get("source_rule_ids")
        .and_then(|v| v.as_sequence())
        .expect("has_sql fixture must list source_rule_ids")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(ids, vec!["2.4", "8.3", "13.1"]);

    // `has_auth.yaml` samples the compound case: `13.3` requires
    // `has_auth` AND `has_web_ui` (activation lives in `requires`).
    let has_auth = parse_yaml(&fixtures_dir().join("has_auth.yaml"));
    let rule_13_3 = rules_sequence(&has_auth, &fixtures_dir().join("has_auth.yaml"))
        .into_iter()
        .find(|r| r.get("id").and_then(|v| v.as_str()) == Some("13.3"))
        .expect("has_auth fixture must contain rule 13.3");
    let requires: Vec<(&str, bool)> = rule_13_3
        .get("requires")
        .and_then(|v| v.as_sequence())
        .expect("13.3 must carry `requires`")
        .iter()
        .map(|r| {
            (
                r.get("flag").and_then(|v| v.as_str()).unwrap_or(""),
                r.get("equals").and_then(|v| v.as_bool()).unwrap_or(false),
            )
        })
        .collect();
    assert!(requires.contains(&("has_auth", true)));
    assert!(requires.contains(&("has_web_ui", true)));

    // `public_api_and_realtime.yaml` samples the canonical/reference pair:
    // `11.4` lives in `public_api`, `realtime.yaml` only references it by ID.
    let text = fs::read_to_string(fixtures_dir().join("public_api_and_realtime.yaml")).unwrap();
    assert!(
        text.contains("profile_flag: realtime") && text.contains("- ref: '11.4'"),
        "fixture must contain the realtime reference-only section"
    );
}

/// Mirrors `python3 rules/partition_rules.py --check` over the committed
/// reference outputs: per-flag canonical counts, 106 total, 0 duplicated
/// IDs, `realtime` reference-only.
#[test]
fn reference_profiles_match_partition_check() {
    let dir = reference_profiles_dir();
    let expected: HashMap<&str, usize> = HashMap::from([
        ("transversal.yaml", 78),
        ("has_sql.yaml", 3),
        ("has_web_ui.yaml", 6),
        ("has_auth.yaml", 7),
        ("public_api.yaml", 11),
        ("cpu_intensive.yaml", 1),
    ]);

    let mut all_ids: Vec<String> = Vec::new();
    for (file, want) in &expected {
        let path = dir.join(file);
        assert!(
            path.is_file(),
            "missing reference output: {}",
            path.display()
        );
        let doc = parse_yaml(&path);
        assert_eq!(
            doc.get("version").and_then(|v| v.as_u64()),
            Some(2),
            "{file} must carry `version: 2`"
        );
        let rules = rules_sequence(&doc, &path);
        assert_eq!(
            rules.len(),
            *want,
            "{file}: expected {want} canonical rules"
        );
        for rule in &rules {
            let id = rule
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("{}: every canonical rule needs `id`", path.display()));
            all_ids.push(id.to_string());
        }
    }

    let total: usize = expected.values().sum();
    assert_eq!(total, 106);
    assert_eq!(all_ids.len(), 106);
    let unique: HashSet<&String> = all_ids.iter().collect();
    assert_eq!(unique.len(), 106, "0 duplicated IDs across canonical files");

    // `realtime.yaml` contributes no canonical content: entries are
    // `{ref, canonical_file}` pointers, and `11.4` resolves to public_api.
    let realtime = parse_yaml(&dir.join("realtime.yaml"));
    let refs = rules_sequence(&realtime, &dir.join("realtime.yaml"));
    assert!(
        !refs.is_empty(),
        "realtime.yaml must reference at least one rule"
    );
    for entry in &refs {
        assert!(
            entry.get("ref").and_then(|v| v.as_str()).is_some(),
            "realtime.yaml entries must be `ref:` pointers, not rule content"
        );
        assert!(
            entry.get("id").is_none(),
            "realtime.yaml must not duplicate canonical rule bodies"
        );
    }
    let ref_ids: Vec<&str> = refs
        .iter()
        .filter_map(|e| e.get("ref").and_then(|v| v.as_str()))
        .collect();
    assert!(
        ref_ids.contains(&"11.4"),
        "realtime.yaml must reference 11.4"
    );
    assert!(
        all_ids.contains(&"11.4".to_string()),
        "11.4's canonical body must live in public_api.yaml"
    );
}
