//! Pure resolver for the rules subsystem (REQ-rules-004).
//!
//! - [`resolve_active_ruleset`]: single evaluation path over `Rule::requires`
//!   (empty = transversal, non-empty = all entries must match, AND).
//!   `primary_flag` is never consulted here — it is file-organization
//!   metadata used only by [`group_by_primary_flag`] when materializing.
//! - [`group_by_primary_flag`]: groups resolved rules by `primary_flag`
//!   (`None` = transversal) so `spec init`/`profile update` know in which
//!   file to write each rule. Never intervenes in activation.
//! - [`dedup_by_id`]: collapses duplicate rule IDs (defensive today — the
//!   catalog ships unique IDs — mandatory if a rule ever becomes reachable
//!   through two paths).
//!
//! Precedence: catalog → profile → `disabled` (subtract) → `severity_override`
//! (replace severity only). The resolver never touches disk: on-disk YAMLs
//! are a readable cache, never the resolution source.

use std::collections::HashMap;

use super::model::{ProfileAnswers, Rule, Severity};

/// Resolution-time customization: IDs excluded from the active set plus
/// forced severities (content untouched). Mirrors [`super::model::RulesConfig`]
/// without owning it so tests can build options inline.
pub struct ResolveOptions<'a> {
    /// Rule IDs excluded from the active set.
    pub disabled: &'a [String],
    /// Forced severities, keeping the rest of the rule content.
    pub severity_override: &'a HashMap<String, Severity>,
}

/// Compute the active ruleset from the full catalog and the project profile.
///
/// Deterministic: same inputs always yield the same output, sorted by
/// severity (blockers first) with ID order stable inside each severity.
pub fn resolve_active_ruleset(
    all_rules: &[Rule],
    profile: &ProfileAnswers,
    opts: &ResolveOptions,
) -> Vec<Rule> {
    let flags = profile.as_flag_map();

    // Single evaluation path: `requires` empty = transversal,
    // non-empty = every condition must hold (AND).
    // `primary_flag` is NOT consulted here.
    let mut result: Vec<Rule> = all_rules
        .iter()
        .filter(|r| {
            r.requires
                .iter()
                .all(|req| *flags.get(&req.flag).unwrap_or(&false) == req.equals)
        })
        .filter(|r| !opts.disabled.contains(&r.id))
        .cloned()
        .collect();

    for rule in result.iter_mut() {
        if let Some(sev) = opts.severity_override.get(&rule.id) {
            rule.severity = *sev;
        }
    }

    dedup_by_id(&mut result);

    result.sort_by_key(|r| match r.severity {
        Severity::Must => 0,
        Severity::Should => 1,
        Severity::Avoid => 2,
    });
    result
}

/// Collapse duplicate rule IDs, keeping the first occurrence in ID order.
///
/// Sorts by `id` first so duplicates are adjacent for `dedup_by`.
pub fn dedup_by_id(rules: &mut Vec<Rule>) {
    rules.sort_by(|a, b| a.id.cmp(&b.id));
    rules.dedup_by(|a, b| a.id == b.id);
}

/// Group resolved rules by `primary_flag` for materialization.
///
/// Used ONLY by `spec init`/`profile update` to decide in which file each
/// materialized rule goes — never in activation.
pub fn group_by_primary_flag(rules: &[Rule]) -> HashMap<Option<String>, Vec<Rule>> {
    let mut groups: HashMap<Option<String>, Vec<Rule>> = HashMap::new();
    for r in rules {
        groups
            .entry(r.primary_flag.clone())
            .or_default()
            .push(r.clone());
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::super::model::load_catalog;
    use super::*;

    fn profile_with(
        has_sql: bool,
        public_api: bool,
        has_web_ui: bool,
        has_auth: bool,
        realtime: bool,
    ) -> ProfileAnswers {
        ProfileAnswers {
            has_sql,
            public_api,
            has_web_ui,
            has_auth,
            realtime,
            ..Default::default()
        }
    }

    fn empty_profile() -> ProfileAnswers {
        ProfileAnswers::default()
    }

    fn full_profile() -> ProfileAnswers {
        ProfileAnswers {
            handles_pii: true,
            has_sql: true,
            public_api: true,
            has_web_ui: true,
            has_auth: true,
            realtime: true,
            cpu_intensive: true,
            long_lived: true,
        }
    }

    fn default_opts<'a>(overrides: &'a HashMap<String, Severity>) -> ResolveOptions<'a> {
        ResolveOptions {
            disabled: &[],
            severity_override: overrides,
        }
    }

    fn ids(rules: &[Rule]) -> Vec<&str> {
        rules.iter().map(|r| r.id.as_str()).collect()
    }

    #[test]
    fn t1_empty_profile_resolves_78_transversal() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let active =
            resolve_active_ruleset(&catalog.rules, &empty_profile(), &default_opts(&overrides));
        assert_eq!(active.len(), 78);
        assert!(active.iter().all(|r| r.requires.is_empty()));
    }

    #[test]
    fn t2_has_sql_resolves_81_with_expected_ids() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let active = resolve_active_ruleset(
            &catalog.rules,
            &profile_with(true, false, false, false, false),
            &default_opts(&overrides),
        );
        assert_eq!(active.len(), 81);
        let ids = ids(&active);
        for expected in ["2.4", "8.3", "13.1"] {
            assert!(ids.contains(&expected), "{expected} must be active");
        }
    }

    #[test]
    fn t3_has_auth_without_web_ui_excludes_13_3() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let active = resolve_active_ruleset(
            &catalog.rules,
            &profile_with(false, false, false, true, false),
            &default_opts(&overrides),
        );
        assert_eq!(active.len(), 84);
        assert!(!ids(&active).contains(&"13.3"));
    }

    #[test]
    fn t4_has_auth_with_web_ui_includes_13_3() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let active = resolve_active_ruleset(
            &catalog.rules,
            &profile_with(false, false, true, true, false),
            &default_opts(&overrides),
        );
        assert_eq!(active.len(), 91);
        assert!(ids(&active).contains(&"13.3"));
    }

    #[test]
    fn t5_public_api_with_realtime_includes_11_4_once() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let active = resolve_active_ruleset(
            &catalog.rules,
            &profile_with(false, true, false, false, true),
            &default_opts(&overrides),
        );
        assert_eq!(active.len(), 89);
        assert_eq!(
            active.iter().filter(|r| r.id == "11.4").count(),
            1,
            "11.4 must appear exactly once"
        );
    }

    #[test]
    fn t5b_realtime_without_public_api_resolves_78() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let active = resolve_active_ruleset(
            &catalog.rules,
            &profile_with(false, false, false, false, true),
            &default_opts(&overrides),
        );
        assert_eq!(active.len(), 78);
        assert!(!ids(&active).contains(&"11.4"));
    }

    #[test]
    fn t5c_public_api_without_realtime_resolves_88_without_11_4() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let active = resolve_active_ruleset(
            &catalog.rules,
            &profile_with(false, true, false, false, false),
            &default_opts(&overrides),
        );
        assert_eq!(active.len(), 88);
        assert!(!ids(&active).contains(&"11.4"));
    }

    #[test]
    fn t5d_synthetic_duplicate_id_collapses_to_one() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let mut duplicated: Vec<Rule> = catalog.rules.iter().take(3).cloned().collect();
        let mut altered = duplicated[0].clone();
        altered.action = "different content, same id".to_string();
        duplicated.push(altered);
        let active =
            resolve_active_ruleset(&duplicated, &empty_profile(), &default_opts(&overrides));
        // Only transversal rules among the first 3 survive; the duplicated
        // id (whatever it is) must appear exactly once.
        let first_id = catalog.rules[0].id.clone();
        assert_eq!(
            active.iter().filter(|r| r.id == first_id).count(),
            1,
            "dedup_by must collapse the duplicated id"
        );
    }

    #[test]
    fn t6_disabled_id_is_excluded() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let opts = ResolveOptions {
            disabled: &["13.1".to_string()],
            severity_override: &overrides,
        };
        let active = resolve_active_ruleset(
            &catalog.rules,
            &profile_with(true, false, false, false, false),
            &opts,
        );
        assert_eq!(active.len(), 80);
        assert!(!ids(&active).contains(&"13.1"));
    }

    #[test]
    fn t7_severity_override_replaces_severity_only() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::from([("14.5".to_string(), Severity::Must)]);
        let active =
            resolve_active_ruleset(&catalog.rules, &empty_profile(), &default_opts(&overrides));
        let original = catalog.rules.iter().find(|r| r.id == "14.5").unwrap();
        assert_eq!(original.severity, Severity::Should);
        let overridden = active.iter().find(|r| r.id == "14.5").unwrap();
        assert_eq!(overridden.severity, Severity::Must);
        assert_eq!(overridden.action, original.action);
        assert_eq!(overridden.condition, original.condition);
    }

    #[test]
    fn t8_full_profile_resolves_106_sorted_by_severity() {
        let catalog = load_catalog().unwrap();
        let overrides = HashMap::new();
        let active =
            resolve_active_ruleset(&catalog.rules, &full_profile(), &default_opts(&overrides));
        assert_eq!(active.len(), 106);
        let mut seen: Vec<String> = ids(&active).iter().map(|s| s.to_string()).collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 106, "no duplicated IDs");
        let rank = |r: &Rule| match r.severity {
            Severity::Must => 0,
            Severity::Should => 1,
            Severity::Avoid => 2,
        };
        assert!(
            active.windows(2).all(|w| rank(&w[0]) <= rank(&w[1])),
            "blockers (MUST) first, then SHOULD, then AVOID"
        );
    }

    #[test]
    fn t004_partition_groups_match_reference_counts() {
        // Mirrors `python3 rules/partition_rules.py --check` (EXPECTED) over
        // the embedded catalog: per-flag canonical counts, 106 total with
        // unique IDs, and `realtime` contributing no canonical rules
        // (reference-only via `cross_referenced_by`).
        let catalog = load_catalog().unwrap();
        let groups = group_by_primary_flag(&catalog.rules);
        let count = |flag: Option<&str>| {
            groups
                .get(&flag.map(|f| f.to_string()))
                .map(|v| v.len())
                .unwrap_or(0)
        };
        assert_eq!(count(None), 78);
        assert_eq!(count(Some("has_sql")), 3);
        assert_eq!(count(Some("has_web_ui")), 6);
        assert_eq!(count(Some("has_auth")), 7);
        assert_eq!(count(Some("public_api")), 11);
        assert_eq!(count(Some("cpu_intensive")), 1);
        assert_eq!(catalog.rules.len(), 106);
        let mut seen: Vec<&str> = ids(&catalog.rules);
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 106, "0 duplicated IDs in the catalog");
        assert!(
            groups.keys().all(|k| k.is_none()
                || [
                    "has_sql",
                    "has_web_ui",
                    "has_auth",
                    "public_api",
                    "cpu_intensive"
                ]
                .contains(&k.as_deref().unwrap_or(""))),
            "no unknown primary_flag values"
        );
        let realtime_refs: Vec<&Rule> = catalog
            .rules
            .iter()
            .filter(|r| r.cross_referenced_by.contains(&"realtime".to_string()))
            .collect();
        assert!(
            realtime_refs.iter().any(|r| r.id == "11.4"),
            "11.4 must be cross-referenced by realtime"
        );
        assert!(
            realtime_refs
                .iter()
                .all(|r| r.primary_flag.as_deref() != Some("realtime")),
            "realtime owns no canonical rules"
        );
    }
}
