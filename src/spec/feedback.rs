//! Continuous-improvement loop for the spec generator (REQ-012).
//!
//! Three pieces, all declared in `resources/content_profiles.yaml`:
//!
//! 1. **Memory** — `dectl memory add` records generator defects, non-fit items
//!    and decisions per project, so the knowledge survives across sessions and
//!    projects.
//! 2. **Evaluation** — a reusable pass/fail template: compiling and starting is
//!    the generator's promise; business-rule gaps are outside the promise.
//!    Every evaluation reuses it unchanged.
//! 3. **Proposals** — a non-fit item seen in at least two independent projects
//!    is emitted as evidence for the proposal pipeline owned by the emitting
//!    system. dectl keeps no changelog of its own: it only prints the evidence
//!    block, with the item's origin and the projects where it recurred.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;

use crate::bail_app_err;
use crate::core::config::ProjectConfig;
use crate::core::db::{get_db, MemoryEntry, Storage, MEMORY_SELECT_COLS};
use crate::core::output::OutputMode;

/// Prefix that marks a memory entry as a structured non-fit record.
const NON_FIT_PREFIX: &str = "GEN_FEEDBACK_NON_FIT ";

/// Tag shared by every generator-feedback memory entry.
const BASE_TAG: &str = "generator";

/// Embedded catalog, shared with the content-profile module.
#[derive(Debug, Deserialize)]
struct CatalogFile {
    feedback: FeedbackDef,
}

/// The three pieces of the continuous-improvement loop.
#[derive(Debug, Clone, Deserialize)]
pub struct FeedbackDef {
    /// Where and how feedback is recorded.
    pub memory: MemoryDef,
    /// Reusable pass/fail evaluation template.
    pub evaluation: EvaluationDef,
    /// Hand-off of recurring non-fit items to the emitting system.
    pub proposals: ProposalsDef,
}

/// Memory contract: command, scope and tags used to record feedback.
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryDef {
    /// Recording command.
    pub command: String,
    /// Scope of every record (`project`).
    pub scope: String,
    /// One tag per feedback kind.
    pub tags: MemoryTags,
    /// Shape of the content recorded for each kind.
    pub records: Vec<MemoryRecord>,
}

/// Tags for the three feedback kinds.
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryTags {
    /// Defects found in the generator output.
    pub defect: String,
    /// Structured non-fit items.
    pub non_fit: String,
    /// Decisions taken about the generator.
    pub decision: String,
}

/// One documented memory record shape.
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryRecord {
    /// Feedback kind (`defect`, `non_fit`, `decision`).
    pub kind: String,
    /// What the entry must contain.
    pub content: String,
}

/// Reusable pass/fail evaluation template.
#[derive(Debug, Clone, Deserialize)]
pub struct EvaluationDef {
    /// Template name.
    pub name: String,
    /// Template version.
    pub version: u32,
    /// Promise the generator makes.
    pub promise: String,
    /// Criteria, each marked inside or outside the promise.
    pub criteria: Vec<EvaluationCriterion>,
    /// Reuse rule, so future evaluations do not fork the template.
    pub reuse: String,
}

/// One evaluation criterion.
#[derive(Debug, Clone, Deserialize)]
pub struct EvaluationCriterion {
    /// Stable criterion id.
    pub id: String,
    /// `inside` or `outside` the promise.
    pub promise: String,
    /// What the criterion checks.
    pub check: String,
}

/// Proposal-pipeline hand-off contract.
#[derive(Debug, Clone, Deserialize)]
pub struct ProposalsDef {
    /// System that owns the proposal pipeline.
    pub owner: String,
    /// Reference to the owner's schema declaration.
    pub schema_ref: String,
    /// Number of independent projects that makes an item recurring.
    pub recurrence_threshold: usize,
    /// Fields the evidence block must carry.
    pub evidence_fields: Vec<String>,
    /// Whether dectl keeps its own changelog (must stay false).
    pub own_changelog: bool,
}

/// Feedback kind accepted by `dectl spec feedback --kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackKind {
    /// A defect found in the generator output.
    Defect,
    /// An item that does not map to a declarative spec entry.
    NonFit,
    /// A decision taken about the generator.
    Decision,
}

impl FeedbackKind {
    /// Parse the CLI value.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "defect" => Some(Self::Defect),
            "non-fit" | "non_fit" => Some(Self::NonFit),
            "decision" => Some(Self::Decision),
            _ => None,
        }
    }

    /// Memory type stored for this kind.
    fn memory_type(self) -> &'static str {
        match self {
            Self::Decision => "decision",
            Self::Defect | Self::NonFit => "note",
        }
    }
}

fn catalog() -> &'static FeedbackDef {
    static CATALOG: OnceLock<FeedbackDef> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let file: CatalogFile = serde_yaml::from_str(super::content_profile::CONTENT_PROFILES_YAML)
            .expect("embedded resources/content_profiles.yaml must be valid");
        file.feedback
    })
}

/// The declared continuous-improvement loop.
pub fn feedback() -> &'static FeedbackDef {
    catalog()
}

/// Memory tag for a feedback kind.
pub fn tag_for(kind: FeedbackKind) -> &'static str {
    let tags = &feedback().memory.tags;
    match kind {
        FeedbackKind::Defect => &tags.defect,
        FeedbackKind::NonFit => &tags.non_fit,
        FeedbackKind::Decision => &tags.decision,
    }
}

/// Render the reusable pass/fail evaluation template.
///
/// The output is deterministic: future evaluations reuse it unchanged.
pub fn render_evaluation_template() -> String {
    let eval = &feedback().evaluation;
    let mut out = format!(
        "# Generator Evaluation — pass/fail template\n\n\
         > Reusable and versioned. Future evaluations reuse this template unchanged; \
         do not fork it per project.\n\n\
         **Template**: {} v{}\n\
         **Promise**: {}\n\n\
         | Criterion | Promise | Passes when |\n\
         |---|---|---|\n",
        eval.name, eval.version, eval.promise
    );
    for criterion in &eval.criteria {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            criterion.id, criterion.promise, criterion.check
        ));
    }
    out.push_str(
        "\n**Verdict rule**: every `inside` criterion is the promise — if one fails, the \
         evaluation FAILS. `outside` criteria are recorded as gaps and follow-ups; they do \
         not break the promise, but they feed the proposal pipeline.\n",
    );
    out.push_str(&format!("\n**Reuse**: {}\n", eval.reuse));
    out
}

/// Guidance block appended to the spec-generation instructions.
pub fn render_guidance_block() -> String {
    let loop_def = feedback();
    let mut out = format!(
        "[GENERATOR FEEDBACK LOOP]\n\
         Record every generator defect, non-fit item and decision in project memory with \
         `{}` (scope: {}):\n",
        loop_def.memory.command, loop_def.memory.scope
    );
    for record in &loop_def.memory.records {
        out.push_str(&format!(
            "- {} [tag `{}`] — {}\n",
            record.kind,
            tag_for(FeedbackKind::parse(&record.kind).unwrap_or(FeedbackKind::Defect)),
            record.content
        ));
    }
    let eval = &loop_def.evaluation;
    out.push_str(&format!(
        "Evaluation: reuse the `{}` v{} template unchanged (compiles and starts = the \
         promise; business-rule gaps = outside the promise).\n",
        eval.name, eval.version
    ));
    let proposals = &loop_def.proposals;
    out.push_str(&format!(
        "Recurring non-fit: an item present in at least {} independent projects MUST be fed \
         as evidence to the {} proposal pipeline ({}) — run `dectl spec feedback --report`. \
         dectl keeps no local changelog.\n",
        proposals.recurrence_threshold, proposals.owner, proposals.schema_ref
    ));
    out.push_str("[/GENERATOR FEEDBACK LOOP]");
    out
}

/// Write the evaluation template into a project, without overwriting one that
/// already exists (a filled evaluation must stay unchanged).
pub fn write_evaluation_template(project_dir: &Path) -> Result<()> {
    let path = project_dir
        .join(".dec")
        .join("sdd")
        .join("generator-evaluation.md");
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, render_evaluation_template())?;
    Ok(())
}

/// A structured non-fit observation stored in project memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonFitRecord {
    /// Stable identifier that survives regeneration.
    pub id: String,
    /// Project where the item was observed.
    pub project: String,
    /// Why the element does not map directly to a declarative entry.
    pub reason: String,
    /// Section of the source instructions the item comes from.
    pub origin: String,
}

impl NonFitRecord {
    /// Serialize the record into the memory content format.
    pub fn to_memory_content(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => format!("{NON_FIT_PREFIX}{json}"),
            Err(_) => format!("{NON_FIT_PREFIX}{{}}"),
        }
    }

    /// Parse a memory entry; returns `None` when the entry is not a non-fit
    /// record or carries no project scope.
    pub fn from_memory(content: &str, project: Option<&str>) -> Option<Self> {
        let json = content.strip_prefix(NON_FIT_PREFIX)?;
        let mut record: Self = serde_json::from_str(json).ok()?;
        if let Some(project) = project {
            record.project = project.to_string();
        }
        if record.project.trim().is_empty() || record.id.trim().is_empty() {
            return None;
        }
        Some(record)
    }
}

/// A non-fit item observed in enough independent projects to be recurring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecurringNonFit {
    /// Stable identifier shared by every observation.
    pub id: String,
    /// Distinct projects where it appeared, sorted.
    pub projects: Vec<String>,
    /// Distinct reasons recorded.
    pub reasons: Vec<String>,
    /// Distinct origins recorded.
    pub origins: Vec<String>,
}

/// Distinct projects, reasons and origins collected for one stable id.
type ObservationSets<'a> = (BTreeSet<&'a str>, BTreeSet<&'a str>, BTreeSet<&'a str>);

/// Group non-fit observations by stable id and keep the recurring ones.
///
/// An item recurs when it appears in at least the declared threshold of
/// independent projects; repeated observations inside one project do not count.
pub fn recurring(records: &[NonFitRecord]) -> Vec<RecurringNonFit> {
    let threshold = feedback().proposals.recurrence_threshold.max(2);
    let mut grouped: BTreeMap<&str, ObservationSets> = BTreeMap::new();

    for record in records {
        if record.project.trim().is_empty() {
            continue;
        }
        let entry = grouped.entry(record.id.as_str()).or_default();
        entry.0.insert(record.project.as_str());
        entry.1.insert(record.reason.as_str());
        entry.2.insert(record.origin.as_str());
    }

    grouped
        .into_iter()
        .filter(|(_, (projects, _, _))| projects.len() >= threshold)
        .map(|(id, (projects, reasons, origins))| RecurringNonFit {
            id: id.to_string(),
            projects: projects.into_iter().map(str::to_string).collect(),
            reasons: reasons.into_iter().map(str::to_string).collect(),
            origins: origins.into_iter().map(str::to_string).collect(),
        })
        .collect()
}

/// Render the evidence block handed to the emitting system's proposal pipeline.
///
/// Nothing is written to disk: dectl does not keep a parallel proposal record.
pub fn render_proposal_evidence(items: &[RecurringNonFit]) -> String {
    let proposals = &feedback().proposals;
    let mut out = format!(
        "[NON-FIT PROPOSAL EVIDENCE]\n\
         Proposal pipeline: {} ({})\n\
         Recurrence threshold: {} independent projects\n\
         Evidence fields: {}\n",
        proposals.owner,
        proposals.schema_ref,
        proposals.recurrence_threshold,
        proposals.evidence_fields.join(", ")
    );
    if items.is_empty() {
        out.push_str("- none\n");
    } else {
        for item in items {
            out.push_str(&format!(
                "- {} — projects: {} — reason: {} — origin: {}\n",
                item.id,
                item.projects.join(", "),
                item.reasons.join("; "),
                item.origins.join("; ")
            ));
        }
    }
    if proposals.own_changelog {
        out.push_str("Feed this block to the emitting system's proposal pipeline as evidence.\n");
    } else {
        out.push_str(
            "Feed this block to the emitting system's proposal pipeline as evidence. \
             dectl keeps no local changelog.\n",
        );
    }
    out.push_str("[/NON-FIT PROPOSAL EVIDENCE]");
    out
}

/// Report printed by `dectl spec feedback --report`.
#[derive(Debug, Serialize)]
pub struct FeedbackReport {
    /// System that owns the proposal pipeline.
    pub owner: String,
    /// Reference to that system's schema declaration.
    pub schema_ref: String,
    /// Declared recurrence threshold.
    pub threshold: usize,
    /// Non-fit items that reached the threshold.
    pub recurring: Vec<RecurringNonFit>,
    /// Ready-to-feed evidence block.
    pub evidence: String,
}

/// Arguments for `dectl spec feedback`.
pub struct FeedbackArgs {
    /// Kind of feedback to record; `None` prints the loop contract.
    pub kind: Option<FeedbackKind>,
    /// Content for `defect` and `decision`; reason for `non-fit`.
    pub content: Option<String>,
    /// Stable id, required for `non-fit`.
    pub id: Option<String>,
    /// Source section, required for `non-fit`.
    pub origin: Option<String>,
    /// Print the recurrence report instead of recording.
    pub report: bool,
    /// JSON output.
    pub json: bool,
}

/// Run the feedback subcommand.
pub fn run(args: FeedbackArgs) -> Result<()> {
    let mode = OutputMode::from_json_flag(args.json);
    if args.report {
        return report(mode);
    }
    match args.kind {
        Some(kind) => record(kind, args.content, args.id, args.origin, mode),
        None => show(mode),
    }
}

fn show(mode: OutputMode) -> Result<()> {
    if mode.is_json() {
        let envelope = serde_json::json!({
            "status": "ok",
            "data": {
                "memory": {
                    "command": feedback().memory.command,
                    "scope": feedback().memory.scope,
                    "tags": {
                        "defect": tag_for(FeedbackKind::Defect),
                        "non_fit": tag_for(FeedbackKind::NonFit),
                        "decision": tag_for(FeedbackKind::Decision),
                    },
                },
                "evaluation": feedback().evaluation.name,
                "evaluation_version": feedback().evaluation.version,
                "evaluation_template": render_evaluation_template(),
                "proposal_owner": feedback().proposals.owner,
                "recurrence_threshold": feedback().proposals.recurrence_threshold,
            }
        });
        println!("{}", serde_json::to_string_pretty(&envelope)?);
        return Ok(());
    }

    println!("{}", render_guidance_block());
    println!();
    println!("{}", render_evaluation_template());
    Ok(())
}

fn record(
    kind: FeedbackKind,
    content: Option<String>,
    id: Option<String>,
    origin: Option<String>,
    mode: OutputMode,
) -> Result<()> {
    let content = content.unwrap_or_default();
    if content.trim().is_empty() {
        bail_app_err!(
            "Feedback content is empty",
            "Use: dectl spec feedback --kind <defect|non-fit|decision> --content \"...\""
        );
    }

    let project = ProjectConfig::current_project_name().ok_or_else(|| {
        anyhow::anyhow!(
            "Cannot resolve the current project: .dec/config/project.toml is missing or invalid"
        )
    })?;

    let body = match kind {
        FeedbackKind::Defect => format!("Generator defect: {}", content.trim()),
        FeedbackKind::Decision => format!("Generator decision: {}", content.trim()),
        FeedbackKind::NonFit => {
            let id = id.unwrap_or_default();
            let origin = origin.unwrap_or_default();
            if id.trim().is_empty() || origin.trim().is_empty() {
                bail_app_err!(
                    "Non-fit feedback requires a stable id and an origin",
                    "Use: dectl spec feedback --kind non-fit --id <id> --origin <section> --content <reason>"
                );
            }
            NonFitRecord {
                id: id.trim().to_string(),
                project: project.clone(),
                reason: content.trim().to_string(),
                origin: origin.trim().to_string(),
            }
            .to_memory_content()
        }
    };

    let tags = format!("{BASE_TAG},{}", tag_for(kind));
    crate::memory::add::run(
        Some(body),
        Some(tags),
        Some(project),
        kind.memory_type().to_string(),
        mode,
    )?;

    if kind == FeedbackKind::NonFit && !mode.is_json() {
        println!("Non-fit recorded. Check recurrence with: dectl spec feedback --report");
    }

    Ok(())
}

fn report(mode: OutputMode) -> Result<()> {
    let records = load_non_fit_records()?;
    let recurring = recurring(&records);
    let proposals = &feedback().proposals;
    let report = FeedbackReport {
        owner: proposals.owner.clone(),
        schema_ref: proposals.schema_ref.clone(),
        threshold: proposals.recurrence_threshold,
        evidence: render_proposal_evidence(&recurring),
        recurring,
    };

    if mode.is_json() {
        mode.print(&report)?;
    } else {
        println!("{}", report.evidence);
    }
    Ok(())
}

fn load_non_fit_records() -> Result<Vec<NonFitRecord>> {
    let db = get_db()?;
    let like = format!("%{}%", tag_for(FeedbackKind::NonFit));
    let sql = format!(
        "SELECT {} FROM memories WHERE deleted_at IS NULL AND tags LIKE ?1",
        MEMORY_SELECT_COLS
    );
    let entries = db.query_map(
        &sql,
        &[&like as &dyn rusqlite::types::ToSql],
        MemoryEntry::from_row,
    )?;

    Ok(entries
        .into_iter()
        .filter_map(|entry| NonFitRecord::from_memory(&entry.content, entry.project.as_deref()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str, project: &str) -> NonFitRecord {
        NonFitRecord {
            id: id.to_string(),
            project: project.to_string(),
            reason: "rule does not map".to_string(),
            origin: "§2.1".to_string(),
        }
    }

    #[test]
    fn test_feedback_catalog_declares_memory_tags_and_scope() {
        let loop_def = feedback();
        assert_eq!(loop_def.memory.command, "dectl memory add");
        assert_eq!(loop_def.memory.scope, "project");
        assert_eq!(tag_for(FeedbackKind::Defect), "generator-defect");
        assert_eq!(tag_for(FeedbackKind::NonFit), "generator-non-fit");
        assert_eq!(tag_for(FeedbackKind::Decision), "generator-decision");
        assert_eq!(loop_def.memory.records.len(), 3);
    }

    #[test]
    fn test_evaluation_template_is_pass_fail_and_reusable() {
        let eval = &feedback().evaluation;
        assert_eq!(eval.name, "generator-pass-fail");
        let inside: Vec<&str> = eval
            .criteria
            .iter()
            .filter(|c| c.promise == "inside")
            .map(|c| c.id.as_str())
            .collect();
        let outside: Vec<&str> = eval
            .criteria
            .iter()
            .filter(|c| c.promise == "outside")
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(inside, vec!["compiles", "starts"]);
        assert_eq!(outside, vec!["business_rule"]);

        let template = render_evaluation_template();
        assert!(template.contains("pass/fail"));
        assert!(template.contains("| compiles | inside |"));
        assert!(template.contains("| business_rule | outside |"));
        assert!(template.contains("evaluation FAILS"));
        assert!(template.contains(&eval.reuse));
    }

    #[test]
    fn test_non_fit_record_round_trips_through_memory_content() {
        let original = record("NF-1", "ledger");
        let content = original.to_memory_content();
        assert!(content.starts_with(NON_FIT_PREFIX));
        let parsed = NonFitRecord::from_memory(&content, None).unwrap();
        assert_eq!(parsed, original);
        assert_eq!(NonFitRecord::from_memory("plain note", Some("p")), None);
    }

    #[test]
    fn test_recurring_requires_independent_projects() {
        let records = vec![
            record("NF-1", "ledger"),
            record("NF-1", "ledger"),
            record("NF-1", "clinic"),
            record("NF-2", "ledger"),
        ];

        let items = recurring(&records);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "NF-1");
        assert_eq!(items[0].projects, vec!["clinic", "ledger"]);
    }

    #[test]
    fn test_recurring_ignores_unscoped_records() {
        let records = vec![record("NF-9", ""), record("NF-9", "ledger")];
        assert!(recurring(&records).is_empty());
    }

    #[test]
    fn test_proposal_evidence_names_owner_and_keeps_no_local_changelog() {
        let evidence = render_proposal_evidence(&recurring(&[
            record("NF-1", "ledger"),
            record("NF-1", "clinic"),
        ]));
        assert!(evidence.contains("[NON-FIT PROPOSAL EVIDENCE]"));
        assert!(evidence.contains(&feedback().proposals.owner));
        assert!(evidence.contains(&feedback().proposals.schema_ref));
        assert!(evidence.contains("NF-1 — projects: clinic, ledger"));
        assert!(evidence.contains("dectl keeps no local changelog"));
    }

    #[test]
    fn test_proposal_contract_has_no_own_changelog() {
        assert!(!feedback().proposals.own_changelog);
        assert_eq!(feedback().proposals.recurrence_threshold, 2);
        assert!(feedback()
            .proposals
            .evidence_fields
            .iter()
            .any(|field| field == "origin"));
    }

    #[test]
    fn test_guidance_block_covers_the_three_pieces() {
        let block = render_guidance_block();
        assert!(block.contains("[GENERATOR FEEDBACK LOOP]"));
        assert!(block.contains("dectl memory add"));
        assert!(block.contains("generator-non-fit"));
        assert!(block.contains("generator-pass-fail"));
        assert!(block.contains("dectl spec feedback --report"));
        assert!(block.contains("dectl keeps no local changelog"));
    }
}
