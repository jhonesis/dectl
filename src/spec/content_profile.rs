//! Content-profile selection for spec generation (REQ-012).
//!
//! A content profile narrows the generated spec suite to the documents a
//! consuming system declares. The complete suite stays the default, so projects
//! that do not opt in are unaffected.
//!
//! Precedence, highest first:
//! 1. the explicit `--profile <name>` option;
//! 2. a `profile: <name>` declaration in the YAML front matter of the `--from`
//!    source file;
//! 3. the complete document suite (no profile).
//!
//! Source-file header example:
//!
//! ```text
//! ---
//! profile: core
//! ---
//! # Requirements
//! ```
//!
//! Declared profiles live in `resources/content_profiles.yaml`, embedded at
//! compile time. A profile declares the consuming system, the documents it
//! reads (with the shape each must have) and the documents left out with the
//! reason. The generated guidance lists both sides, so a reduced profile is
//! auditable instead of implicit.

use anyhow::Result;
use serde::Deserialize;
use std::sync::OnceLock;

use crate::bail_app_err;

/// Embedded catalog of declared content profiles.
pub(crate) const CONTENT_PROFILES_YAML: &str =
    include_str!("../../resources/content_profiles.yaml");

/// Declared profile catalog, parsed once from the embedded YAML.
#[derive(Debug, Deserialize)]
struct ProfilesFile {
    #[allow(dead_code)] // version is asserted by tests; keep the file self-describing
    version: u32,
    profiles: Vec<ProfileDef>,
}

/// A content profile declaration.
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileDef {
    /// Profile name as used by `--profile`.
    pub name: String,
    /// One-line statement of why the profile exists.
    pub description: String,
    /// System that consumes the generated documents.
    pub consumer: ConsumerDef,
    /// Scopes where the profile applies (`feature`, `module`).
    #[serde(default)]
    pub scopes: Vec<String>,
    /// Documents the profile generates.
    pub includes: Vec<IncludedDoc>,
    /// Documents the profile deliberately does not generate.
    pub excludes: Vec<ExcludedDoc>,
    /// Mechanical checklist and non-fit rules for the generated contract.
    #[serde(default)]
    pub contract: Option<ContractDef>,
}

/// Auditable contract checklist plus the non-fit schema it consumes.
#[derive(Debug, Clone, Deserialize)]
pub struct ContractDef {
    /// One-line purpose of the checklist.
    pub description: String,
    /// One row per mapped element type.
    pub checklist: Vec<ChecklistItem>,
    /// Non-fit schema, owned by the system that emits the items.
    pub non_fit: NonFitDef,
    /// Required implementation pattern and mandatory test battery.
    pub implementation_guide: ImplementationGuide,
}

/// A single checklist row: one mapped source element type.
#[derive(Debug, Clone, Deserialize)]
pub struct ChecklistItem {
    /// Source element type (entity, endpoint, rule, access, ...).
    pub item: String,
    /// Destination document or section in the generated contract.
    pub destination: String,
    /// What counts as mapped for this row.
    pub check: String,
}

/// Non-fit item schema declared by the emitting system.
///
/// dectl consumes this declaration; it does not define a competing format.
#[derive(Debug, Clone, Deserialize)]
pub struct NonFitDef {
    /// System that emits the items and owns the format.
    pub owner: String,
    /// Reference to the emitter's schema declaration.
    pub schema_ref: String,
    /// File the emitter writes the items to.
    pub file: String,
    /// Fields of the emitter's item schema, in declaration order.
    pub fields: Vec<NonFitField>,
}

/// One field of the emitter's non-fit item schema.
#[derive(Debug, Clone, Deserialize)]
pub struct NonFitField {
    /// Field name as declared by the emitter.
    pub name: String,
    /// What the field records.
    pub meaning: String,
}

/// Required pattern and test battery for implementing a non-fit item.
#[derive(Debug, Clone, Deserialize)]
pub struct ImplementationGuide {
    /// Pattern the implementation must follow.
    pub pattern: String,
    /// Tests that must accompany the implementation.
    pub mandatory_tests: Vec<String>,
    /// Coverage target for the domain layer.
    pub coverage: String,
}

/// Declared consumer of a profile's document set.
#[derive(Debug, Clone, Deserialize)]
pub struct ConsumerDef {
    /// Consuming system name.
    pub system: String,
    /// File where the consumer implements the contract.
    pub entry_point: String,
    /// Documents the consumer reads, verbatim.
    #[serde(default)]
    pub reads: Vec<String>,
}

/// Document included in a profile.
#[derive(Debug, Clone, Deserialize)]
pub struct IncludedDoc {
    /// Path relative to the project root.
    pub path: String,
    /// Consumer of this document; a document without one cannot be included.
    pub consumer: String,
    /// Shape the document must have to be consumed without adapters.
    pub shape: String,
}

/// Document excluded from a profile.
#[derive(Debug, Clone, Deserialize)]
pub struct ExcludedDoc {
    /// Path relative to the project specs root.
    pub path: String,
    /// Why the profile leaves it out.
    pub reason: String,
}

fn catalog() -> &'static ProfilesFile {
    static CATALOG: OnceLock<ProfilesFile> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_yaml::from_str(CONTENT_PROFILES_YAML)
            .expect("embedded resources/content_profiles.yaml must be valid")
    })
}

/// Look up a declared profile by name.
pub fn declared(name: &str) -> Option<&'static ProfileDef> {
    catalog()
        .profiles
        .iter()
        .find(|profile| profile.name == name)
}

/// Names of the profiles declared by this build.
pub fn available_names() -> Vec<&'static str> {
    catalog()
        .profiles
        .iter()
        .map(|profile| profile.name.as_str())
        .collect()
}

/// Selected content profile for a spec generation run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentProfile {
    /// No profile selected: generate the complete document suite.
    Full,
    /// Generate only the documents declared by the named profile.
    Named(String),
}

impl ContentProfile {
    /// Profile name, or `None` when the complete suite is selected.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Full => None,
            Self::Named(name) => Some(name),
        }
    }

    /// Guidance marker appended to the generation instructions.
    ///
    /// Returns `None` for the complete suite so default output stays unchanged.
    /// The marker starts with two newlines so it can be appended inline to the
    /// end of a guidance heading without altering the default rendering.
    pub fn marker(&self) -> Option<String> {
        self.name().map(render_marker)
    }
}

/// Render the guidance block for a profile name.
///
/// Declared profiles list the consumer, every included document with its shape
/// and every excluded document with its reason. Undeclared names stay
/// non-fatal: they are reported as unresolved and the complete suite is kept.
fn render_marker(name: &str) -> String {
    match declared(name) {
        Some(profile) => render_declared(profile),
        None => format!(
            "\n\n[CONTENT PROFILE: {name}]\n\
             No content profile named \"{name}\" is declared by this dectl build \
             (available: {}).\n\
             Generate the complete document suite and report the selection as unresolved.\n\
             [/CONTENT PROFILE]",
            available_names().join(", ")
        ),
    }
}

fn render_declared(profile: &ProfileDef) -> String {
    let mut out = format!(
        "\n\n[CONTENT PROFILE: {}]\n\
         Consumer contract: {} reads these documents verbatim (entry point: {}).\n\
         Why: {}\n",
        profile.name, profile.consumer.system, profile.consumer.entry_point, profile.description
    );

    if !profile.consumer.reads.is_empty() {
        out.push_str("Consumed paths:\n");
        for path in &profile.consumer.reads {
            out.push_str(&format!("- `{path}`\n"));
        }
    }

    if !profile.scopes.is_empty() {
        out.push_str(&format!(
            "Applies to scope: {}. In both scopes write the documents below at the project \
             specs root; do not create a module subdirectory.\n",
            profile.scopes.join(" and ")
        ));
    }

    out.push_str("Generate EXACTLY these documents and nothing else:\n");
    for doc in &profile.includes {
        out.push_str(&format!(
            "- `{}` — consumer: {} — shape: {}\n",
            doc.path, doc.consumer, doc.shape
        ));
    }

    out.push_str("Excluded on purpose — do not create them; report each with its reason:\n");
    for doc in &profile.excludes {
        out.push_str(&format!("- `{}` — {}\n", doc.path, doc.reason));
    }

    if let Some(contract) = &profile.contract {
        render_contract(&mut out, contract);
    }

    // Continuous-improvement loop (REQ-012): memory, evaluation template and
    // proposal hand-off apply to every generated contract.
    out.push_str(&super::feedback::render_guidance_block());
    out.push('\n');

    out.push_str("[/CONTENT PROFILE]");
    out
}

/// Render the mechanical contract checklist and the non-fit guide.
fn render_contract(out: &mut String, contract: &ContractDef) {
    out.push_str(&format!(
        "Spec contract checklist — {} Tick one box per mapped element instead of \
         re-reading the source:\n",
        contract.description
    ));
    for row in &contract.checklist {
        out.push_str(&format!(
            "- [ ] {} -> {} — {}\n",
            row.item, row.destination, row.check
        ));
    }

    out.push_str(&format!(
        "Everything that does not map directly goes to the non-fit list; never drop it. \
         The item format is declared by {} ({}), file `{}` — do not define a local format:\n",
        contract.non_fit.owner, contract.non_fit.schema_ref, contract.non_fit.file
    ));
    for field in &contract.non_fit.fields {
        out.push_str(&format!("- `{}` — {}\n", field.name, field.meaning));
    }
    out.push_str(
        "Every non-fit item MUST cite its origin (source section) and its destination \
         (test + code location): traceability origin -> work -> test is mandatory.\n",
    );

    out.push_str("Non-fit implementation guide (required pattern and test battery):\n");
    out.push_str(&format!(
        "- Pattern: {}\n",
        contract.implementation_guide.pattern
    ));
    out.push_str(&format!(
        "- Mandatory tests: {}\n",
        contract.implementation_guide.mandatory_tests.join("; ")
    ));
    out.push_str(&format!(
        "- Coverage target: {}\n",
        contract.implementation_guide.coverage
    ));
}

/// Resolve the content profile from the explicit option and the source header.
///
/// The explicit option wins over the source-file declaration; with neither, the
/// complete suite is returned.
pub fn resolve(explicit: Option<&str>, from_content: Option<&str>) -> Result<ContentProfile> {
    if let Some(name) = explicit {
        return Ok(ContentProfile::Named(validate(name)?));
    }

    if let Some(content) = from_content {
        if let Some(name) = profile_from_source(content) {
            return Ok(ContentProfile::Named(validate(&name)?));
        }
    }

    Ok(ContentProfile::Full)
}

/// Read the `profile:` declaration from a source file's YAML front matter.
///
/// Only a front matter block at the very start of the file is considered, so
/// `profile:` lines in the body are ignored.
pub fn profile_from_source(content: &str) -> Option<String> {
    let trimmed = content.trim_start_matches('\u{feff}').trim_start();
    let mut lines = trimmed.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }

    for line in lines {
        let line = line.trim();
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix("profile:") {
            let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn validate(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        bail_app_err!(
            "Content profile name is empty",
            "Use a profile name, e.g. --profile core"
        );
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail_app_err!(
            format!("Invalid content profile name: '{name}'"),
            "Profile names may contain only letters, digits, '-' and '_'"
        );
    }
    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_without_profile_is_full_suite() {
        assert_eq!(resolve(None, None).unwrap(), ContentProfile::Full);
        assert_eq!(
            resolve(None, Some("# Requirements\n\nNo header here.\n")).unwrap(),
            ContentProfile::Full
        );
        assert!(resolve(None, None).unwrap().marker().is_none());
    }

    #[test]
    fn test_resolve_reads_profile_from_source_header() {
        let source = "---\nprofile: core\n---\n# Requirements\n";
        let profile = resolve(None, Some(source)).unwrap();
        assert_eq!(profile, ContentProfile::Named("core".to_string()));
        assert!(profile.marker().unwrap().contains("CONTENT PROFILE: core"));
    }

    #[test]
    fn test_explicit_option_overrides_source_header() {
        let source = "---\nprofile: core\n---\n# Requirements\n";
        let profile = resolve(Some("lean"), Some(source)).unwrap();
        assert_eq!(profile, ContentProfile::Named("lean".to_string()));
    }

    #[test]
    fn test_profile_body_line_is_ignored() {
        let source = "# Requirements\n\nprofile: core\n";
        assert_eq!(resolve(None, Some(source)).unwrap(), ContentProfile::Full);
    }

    #[test]
    fn test_invalid_profile_name_is_rejected() {
        assert!(resolve(Some(""), None).is_err());
        assert!(resolve(Some("not a profile"), None).is_err());
        assert!(resolve(None, Some("---\nprofile: bad/name\n---\n")).is_err());
    }

    #[test]
    fn test_core_profile_is_declared_with_exact_document_set() {
        let profile = declared("core").expect("core profile must be declared");
        let paths: Vec<&str> = profile
            .includes
            .iter()
            .map(|doc| doc.path.as_str())
            .collect();
        assert_eq!(
            paths,
            vec![
                "specs/spec.md",
                "specs/data-model.md",
                "specs/interface-contracts/cli.md",
            ]
        );
        assert_eq!(profile.consumer.system, "geneia-cli");
        assert!(profile.scopes.contains(&"feature".to_string()));
        assert!(profile.scopes.contains(&"module".to_string()));
    }

    #[test]
    fn test_core_profile_accounts_for_the_complete_suite() {
        let profile = declared("core").expect("core profile must be declared");
        // 15 documents in the SDD suite: 3 included + 12 excluded, no overlap.
        assert_eq!(profile.includes.len() + profile.excludes.len(), 15);

        let mut names: Vec<&str> = profile
            .includes
            .iter()
            .map(|doc| doc.path.rsplit('/').next().unwrap_or_default())
            .chain(profile.excludes.iter().map(|doc| doc.path.as_str()))
            .collect();
        names.sort_unstable();
        let unique = names.len();
        names.dedup();
        assert_eq!(
            names.len(),
            unique,
            "profile must not list a document twice"
        );
    }

    #[test]
    fn test_every_included_document_declares_a_consumer_and_shape() {
        for profile in &catalog().profiles {
            assert!(!profile.description.trim().is_empty(), "{}", profile.name);
            assert!(!profile.consumer.entry_point.trim().is_empty());
            assert!(
                !profile.includes.is_empty(),
                "profile {} must include at least one document",
                profile.name
            );
            for doc in &profile.includes {
                assert!(!doc.path.trim().is_empty());
                assert!(
                    !doc.consumer.trim().is_empty(),
                    "{} must declare a consumer",
                    doc.path
                );
                assert!(
                    !doc.shape.trim().is_empty(),
                    "{} must declare a shape",
                    doc.path
                );
            }
            for doc in &profile.excludes {
                assert!(!doc.path.trim().is_empty());
                assert!(
                    !doc.reason.trim().is_empty(),
                    "{} must declare a reason",
                    doc.path
                );
            }
        }
    }

    #[test]
    fn test_core_marker_lists_included_and_excluded_with_reasons() {
        let marker = resolve(Some("core"), None).unwrap().marker().unwrap();

        assert!(marker.contains("[CONTENT PROFILE: core]"));
        assert!(marker.contains("[/CONTENT PROFILE]"));
        assert!(marker.contains("geneia-cli"));
        assert!(marker.contains("specs/spec.md"));
        assert!(marker.contains("specs/data-model.md"));
        assert!(marker.contains("specs/interface-contracts/cli.md"));
        assert!(marker.contains("Field / Type / Required / Indexes / Description"));
        assert!(marker.contains("### <METHOD> /<path>"));
        assert!(marker.contains("Applies to scope: feature and module"));

        for doc in &declared("core").unwrap().excludes {
            assert!(
                marker.contains(&format!("`{}`", doc.path)) && marker.contains(&doc.reason),
                "excluded document {} must be listed with its reason",
                doc.path
            );
        }

        for doc in &declared("core").unwrap().includes {
            assert!(
                marker.contains(&doc.consumer),
                "included document {} must list its consumer",
                doc.path
            );
        }
    }

    #[test]
    fn test_core_contract_declares_checklist_and_emitter_schema() {
        let profile = declared("core").expect("core profile must be declared");
        let contract = profile
            .contract
            .as_ref()
            .expect("core profile must declare a contract");

        let items: Vec<&str> = contract
            .checklist
            .iter()
            .map(|row| row.item.as_str())
            .collect();
        assert_eq!(items, vec!["entity", "endpoint", "rule", "access"]);

        let destinations: Vec<&str> = contract
            .checklist
            .iter()
            .map(|row| row.destination.as_str())
            .collect();
        assert!(destinations.iter().any(|d| d.contains("data-model.md")));
        assert!(destinations
            .iter()
            .any(|d| d.contains("interface-contracts/cli.md")));
        assert!(destinations.iter().any(|d| d.contains("### Rule")));
        assert!(destinations.iter().any(|d| d.contains("### Access")));

        // The schema belongs to the emitting system: dectl consumes it and
        // must not define a competing format.
        assert_eq!(contract.non_fit.owner, "geneia-cli");
        assert!(!contract.non_fit.schema_ref.trim().is_empty());
        assert_eq!(contract.non_fit.file, "no-fit.yaml");
        let fields: Vec<&str> = contract
            .non_fit
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        assert_eq!(
            fields,
            vec!["id", "reason", "origin", "suggested_location", "test"]
        );

        let guide = &contract.implementation_guide;
        assert!(guide.pattern.contains("business-rule level"));
        assert!(guide.pattern.contains("never edit generated files"));
        assert!(guide
            .mandatory_tests
            .iter()
            .any(|test| test.contains("at least one test per rule")));
        assert!(guide
            .mandatory_tests
            .iter()
            .any(|test| test.contains("concurrency")));
        assert!(guide
            .mandatory_tests
            .iter()
            .any(|test| test.contains("idempotency")));
        assert!(guide.coverage.contains("90%"));
    }

    #[test]
    fn test_core_marker_includes_contract_checklist_and_non_fit_guide() {
        let marker = resolve(Some("core"), None).unwrap().marker().unwrap();

        assert!(marker.contains("Spec contract checklist"));
        assert!(marker.contains("- [ ] entity -> specs/data-model.md"));
        assert!(marker.contains("- [ ] endpoint -> specs/interface-contracts/cli.md"));
        assert!(marker.contains("- [ ] rule -> specs/data-model.md ('### Rule')"));
        assert!(marker.contains("- [ ] access -> specs/data-model.md ('### Access')"));

        assert!(marker.contains("declared by geneia-cli"));
        assert!(marker.contains("no-fit.yaml"));
        assert!(marker.contains("`origin`"));
        assert!(marker.contains("`suggested_location`"));
        assert!(marker.contains("`test`"));
        assert!(marker.contains("origin (source section)"));
        assert!(marker.contains("destination (test + code location)"));

        assert!(marker.contains("Non-fit implementation guide"));
        assert!(marker.contains("business-rule level"));
        assert!(marker.contains("at least one test per rule"));
        assert!(marker.contains("concurrency"));
        assert!(marker.contains("idempotency"));
        assert!(marker.contains("90% in the domain layer"));
        assert!(marker.contains("[GENERATOR FEEDBACK LOOP]"));
        assert!(marker.contains("dectl spec feedback --report"));
    }

    #[test]
    fn test_full_suite_marker_has_no_contract_guidance() {
        assert!(ContentProfile::Full.marker().is_none());
    }

    #[test]
    fn test_undeclared_profile_marker_falls_back_to_complete_suite() {
        let marker = resolve(Some("lean"), None).unwrap().marker().unwrap();
        assert!(marker.contains("[CONTENT PROFILE: lean]"));
        assert!(marker.contains("Generate the complete document suite"));
        assert!(marker.contains("available: core"));
    }
}
