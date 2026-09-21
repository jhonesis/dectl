//! Serde model for the rules subsystem (REQ-rules-002, REQ-rules-004).
//!
//! - [`Severity`], [`RequiredFlag`], [`Rule`], [`RulesFile`]: the embedded
//!   catalog (`RULES_YAML`, schema v2 — activation lives 100% in `requires`,
//!   `primary_flag` is file-organization metadata only).
//! - [`ProfileAnswers`]: the 8 project-feature flags persisted by `spec init`
//!   in `.dec/rules/profile.toml` (`[profile]` + `[profile.meta]`).
//! - [`RulesConfig`]: per-project customization (`[rules]` in `project.toml`:
//!   `disabled` + `severity_override`).
//!
//! Validation is normative: an unknown ID in `disabled`/`severity_override`
//! or an unknown flag in `profile.toml` is an **error naming the ID/flag**
//! (never silently ignored — a typo would leave active a rule the project
//! believed disabled, or drop an answer).

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::data::RULES_YAML;

/// Enforcement level of a rule.
///
/// Maps to reviewer behavior (no effect on the parser): `Must` blocks the
/// pipeline when `action` is not met, `Should` warns, `Avoid` blocks only
/// when the `exception` antipattern is confirmed in the diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Must,
    Should,
    Avoid,
}

/// One activation condition inside `Rule::requires`.
///
/// A rule is active when **all** its entries match the project profile (AND);
/// an empty `requires` list means the rule is transversal (always active).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RequiredFlag {
    pub flag: String,
    pub equals: bool,
}

/// A single development rule from the catalog.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    /// Positional stable ID (`section.index`, e.g. `"13.3"`).
    pub id: String,
    /// Human-readable category name.
    pub section: String,
    pub severity: Severity,
    /// File-organization metadata only — which `.dec/rules/profiles/{flag}.yaml`
    /// materializes this rule. `None` goes to `transversal.yaml`.
    /// Never consulted during activation.
    pub primary_flag: Option<String>,
    /// Activation conditions in AND; empty = always active.
    #[serde(default)]
    pub requires: Vec<RequiredFlag>,
    /// The IF, in plain text.
    pub condition: String,
    /// The THEN — always the recommended practice, regardless of severity.
    pub action: String,
    /// Brief justification, if any.
    pub why: Option<String>,
    /// The antipattern to avoid, if any. Always this role, regardless of
    /// severity.
    pub exception: Option<String>,
    /// Keywords for `dectl rules search`.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Other profiles referencing this rule by ID without duplicating it.
    #[serde(default)]
    pub cross_referenced_by: Vec<String>,
}

/// Root of the embedded catalog (`resources/rules.yaml`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RulesFile {
    /// Catalog version (not the schema version of materialized files).
    pub version: u32,
    pub source: String,
    pub total_rules: usize,
    pub rules: Vec<Rule>,
}

/// Parse the embedded catalog. Fails only if the baked-in YAML drifts out of
/// the schema above (e.g. a catalog edit broke a field).
pub fn load_catalog() -> Result<RulesFile> {
    serde_yaml::from_str(RULES_YAML)
        .map_err(|e| anyhow!("embedded rules catalog failed to parse: {e}"))
}

/// The 8 project-feature flags asked once by `spec init`.
///
/// A question skipped via `depends_on` is persisted as explicit `false` —
/// the resolver never distinguishes "absent" from `false`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ProfileAnswers {
    pub handles_pii: bool,
    pub has_sql: bool,
    pub public_api: bool,
    pub has_web_ui: bool,
    pub has_auth: bool,
    pub realtime: bool,
    pub cpu_intensive: bool,
    pub long_lived: bool,
}

/// Write metadata of `.dec/rules/profile.toml` (`[profile.meta]`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ProfileMeta {
    /// ISO-8601 timestamp, rewritten on every `profile update`.
    pub answered_at: Option<String>,
    /// `"spec_init"` | `"profile_update"`.
    pub answered_via: Option<String>,
    pub schema_version: Option<u32>,
}

/// The 8 flags in questionnaire order (matches `profile.schema.yaml`).
pub const KNOWN_FLAGS: [&str; 8] = [
    "handles_pii",
    "has_sql",
    "public_api",
    "has_web_ui",
    "has_auth",
    "realtime",
    "cpu_intensive",
    "long_lived",
];

impl ProfileAnswers {
    /// View the answers as a flag map for the resolver (`resolve_active_ruleset`).
    pub fn as_flag_map(&self) -> HashMap<String, bool> {
        HashMap::from([
            ("handles_pii".to_string(), self.handles_pii),
            ("has_sql".to_string(), self.has_sql),
            ("public_api".to_string(), self.public_api),
            ("has_web_ui".to_string(), self.has_web_ui),
            ("has_auth".to_string(), self.has_auth),
            ("realtime".to_string(), self.realtime),
            ("cpu_intensive".to_string(), self.cpu_intensive),
            ("long_lived".to_string(), self.long_lived),
        ])
    }

    /// Strictly parse a `.dec/rules/profile.toml` document.
    ///
    /// Unknown flags are an error naming the flag (a typo equals a lost
    /// answer). Returns the answers plus the `[profile.meta]` block.
    pub fn parse_profile_toml(content: &str) -> Result<(Self, ProfileMeta)> {
        let value: toml::Value =
            toml::from_str(content).map_err(|e| anyhow!("profile.toml is not valid TOML: {e}"))?;
        let profile = value
            .get("profile")
            .and_then(|v| v.as_table())
            .ok_or_else(|| anyhow!("profile.toml is missing the [profile] table"))?;
        let mut answers_table = toml::Table::new();
        let mut meta = ProfileMeta::default();
        for (key, val) in profile {
            if key == "meta" {
                meta = val
                    .clone()
                    .try_into()
                    .map_err(|e| anyhow!("profile.toml [profile.meta] is invalid: {e}"))?;
            } else if KNOWN_FLAGS.contains(&key.as_str()) {
                answers_table.insert(key.clone(), val.clone());
            } else {
                return Err(anyhow!(
                    "unknown flag '{key}' in profile.toml [profile] (expected one of: {})",
                    KNOWN_FLAGS.join(", ")
                ));
            }
        }
        let answers: Self = toml::Value::Table(answers_table)
            .try_into()
            .map_err(|e| anyhow!("profile.toml [profile] has a non-boolean flag value: {e}"))?;
        Ok((answers, meta))
    }
}

/// Per-project customization (`[rules]` in `project.toml`).
///
/// ```toml
/// [rules]
/// disabled = ["14.5"]
/// [rules.severity_override]
/// "14.5" = "MUST"   # dotted keys quoted; values UPPERCASE (see [`Severity`])
/// ```
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RulesConfig {
    /// Rule IDs excluded from the active set.
    pub disabled: Vec<String>,
    /// Forced severities, keeping the rest of the rule content.
    pub severity_override: HashMap<String, Severity>,
}

impl RulesConfig {
    /// Validate every ID in `disabled`/`severity_override` against the catalog.
    ///
    /// Unknown IDs are an error naming the ID plus close matches.
    /// Precedence: catalog → profile → `disabled` (subtract) →
    /// `severity_override` (replace severity only).
    pub fn validate(&self, catalog: &[Rule]) -> Result<()> {
        let known: Vec<&str> = catalog.iter().map(|r| r.id.as_str()).collect();
        for id in &self.disabled {
            if !known.contains(&id.as_str()) {
                return Err(unknown_id_error(id, &known, "disabled"));
            }
        }
        for id in self.severity_override.keys() {
            if !known.contains(&id.as_str()) {
                return Err(unknown_id_error(id, &known, "severity_override"));
            }
        }
        Ok(())
    }
}

fn unknown_id_error(id: &str, known: &[&str], field: &str) -> anyhow::Error {
    let mut ranked: Vec<(&str, usize)> = known.iter().map(|k| (*k, edit_distance(id, k))).collect();
    ranked.sort_by_key(|(_, d)| *d);
    let close: Vec<&str> = ranked
        .into_iter()
        .filter(|(_, d)| *d <= 3)
        .take(3)
        .map(|(k, _)| k)
        .collect();
    if close.is_empty() {
        anyhow!("unknown rule id '{id}' in [rules] {field}")
    } else {
        anyhow!(
            "unknown rule id '{id}' in [rules] {field} (did you mean {}?)",
            close.join(", ")
        )
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, &ca) in a.iter().enumerate() {
        let mut curr = vec![i + 1];
        for (j, &cb) in b.iter().enumerate() {
            curr.push(std::cmp::min(
                std::cmp::min(prev[j + 1] + 1, curr[j] + 1),
                prev[j] + usize::from(ca != cb),
            ));
        }
        prev = curr;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_deserializes_106_rules() {
        let catalog = load_catalog().expect("embedded catalog must parse");
        assert_eq!(catalog.rules.len(), 106);
        assert_eq!(catalog.total_rules, catalog.rules.len());
    }

    #[test]
    fn severity_variants_parse_uppercase_only() {
        assert_eq!(
            serde_yaml::from_str::<Severity>("MUST").unwrap(),
            Severity::Must
        );
        assert_eq!(
            serde_yaml::from_str::<Severity>("SHOULD").unwrap(),
            Severity::Should
        );
        assert_eq!(
            serde_yaml::from_str::<Severity>("AVOID").unwrap(),
            Severity::Avoid
        );
        assert!(serde_yaml::from_str::<Severity>("must").is_err());
    }

    #[test]
    fn compound_rules_carry_and_requires() {
        let catalog = load_catalog().unwrap();
        let rule_13_3 = catalog.rules.iter().find(|r| r.id == "13.3").unwrap();
        assert_eq!(rule_13_3.primary_flag.as_deref(), Some("has_auth"));
        assert_eq!(
            rule_13_3.requires,
            vec![
                RequiredFlag {
                    flag: "has_auth".to_string(),
                    equals: true
                },
                RequiredFlag {
                    flag: "has_web_ui".to_string(),
                    equals: true
                },
            ]
        );
        let rule_11_4 = catalog.rules.iter().find(|r| r.id == "11.4").unwrap();
        assert_eq!(rule_11_4.primary_flag.as_deref(), Some("public_api"));
        assert_eq!(rule_11_4.requires.len(), 2);
        assert_eq!(rule_11_4.cross_referenced_by, vec!["realtime".to_string()]);
    }

    #[test]
    fn config_with_known_ids_validates() {
        let catalog = load_catalog().unwrap();
        let config: RulesConfig = toml::from_str(
            "[rules]\ndisabled = [\"14.5\"]\n[rules.severity_override]\n\"14.5\" = \"MUST\"\n",
        )
        .map(|mut wrapper: HashMap<String, RulesConfig>| wrapper.remove("rules"))
        .unwrap()
        .unwrap();
        assert_eq!(config.disabled, vec!["14.5".to_string()]);
        assert_eq!(config.severity_override.get("14.5"), Some(&Severity::Must));
        config.validate(&catalog.rules).unwrap();
    }

    #[test]
    fn config_with_unknown_disabled_id_errors_with_id() {
        let catalog = load_catalog().unwrap();
        let config = RulesConfig {
            disabled: vec!["99.9".to_string()],
            ..Default::default()
        };
        let err = config.validate(&catalog.rules).unwrap_err().to_string();
        assert!(err.contains("99.9"), "error must name the ID: {err}");
    }

    #[test]
    fn config_with_unknown_override_id_suggests_close_match() {
        let catalog = load_catalog().unwrap();
        let config = RulesConfig {
            severity_override: HashMap::from([("13.99".to_string(), Severity::Must)]),
            ..Default::default()
        };
        let err = config.validate(&catalog.rules).unwrap_err().to_string();
        assert!(err.contains("13.99"), "error must name the ID: {err}");
        assert!(
            err.contains("did you mean"),
            "error must suggest close IDs: {err}"
        );
        assert!(
            err.contains("13.9"),
            "13.9 is one edit away from 13.99: {err}"
        );
    }

    #[test]
    fn profile_toml_parses_flags_and_meta() {
        let (answers, meta) = ProfileAnswers::parse_profile_toml(
            "[profile]\nhandles_pii = true\nhas_sql = true\npublic_api = false\n\
             has_web_ui = false\nhas_auth = true\nrealtime = false\n\
             cpu_intensive = false\nlong_lived = true\n\
             [profile.meta]\nanswered_at = \"2026-08-29T12:00:00Z\"\n\
             answered_via = \"spec_init\"\nschema_version = 2\n",
        )
        .unwrap();
        assert!(answers.handles_pii && answers.has_sql && answers.has_auth && answers.long_lived);
        assert!(!answers.public_api && !answers.has_web_ui);
        assert_eq!(meta.answered_via.as_deref(), Some("spec_init"));
        assert_eq!(meta.schema_version, Some(2));
        let flags = answers.as_flag_map();
        assert_eq!(flags.len(), 8);
        assert!(flags["has_sql"]);
        assert!(!flags["realtime"]);
    }

    #[test]
    fn profile_toml_rejects_unknown_flag() {
        let err = ProfileAnswers::parse_profile_toml("[profile]\nhas_sqqq = true\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("has_sqqq"), "error must name the flag: {err}");
    }

    #[test]
    fn profile_toml_missing_table_errors() {
        assert!(ProfileAnswers::parse_profile_toml("[other]\nx = 1\n").is_err());
    }
}
