//! Embedded rules catalog.
//!
//! The YAML sources live in `dectl/resources/` (copied from `rules/`).
//! They are baked into the binary at compile time so every `rules`
//! subcommand works offline with zero external file reads.
//!
//! `RULES_YAML` is wired into `model::load_catalog()` and consumed by
//! `resolver::resolve_active_ruleset`; `PROFILE_SCHEMA_YAML` stays
//! dead-code-allowed until the `spec init` questionnaire parses it.
/// Contents of `resources/rules.yaml` — the 106 machine-readable dev rules.
#[allow(dead_code)] // live via model::load_catalog, consumed by CLI in T005+
pub const RULES_YAML: &str = include_str!("../../resources/rules.yaml");

#[allow(dead_code)]
/// Contents of `resources/profile.schema.yaml` — the profile questionnaire bank.
pub const PROFILE_SCHEMA_YAML: &str = include_str!("../../resources/profile.schema.yaml");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_yaml_is_embedded() {
        assert!(!RULES_YAML.is_empty());
        assert!(RULES_YAML.contains("total_rules: 106"));
    }

    #[test]
    fn profile_schema_yaml_is_embedded() {
        assert!(!PROFILE_SCHEMA_YAML.is_empty());
        // Structural keys (stable across the REQ-007 rewording of the
        // questionnaire copy in T036); never assert Spanish strings here.
        assert!(PROFILE_SCHEMA_YAML.contains("questions:"));
        assert!(PROFILE_SCHEMA_YAML.contains("output:"));
    }
}
