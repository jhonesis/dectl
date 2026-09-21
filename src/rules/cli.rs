//! `dectl rules` subcommands (REQ-rules-003, REQ-rules-006):
//! `list`, `search`, `resolve`, `context`, `profile update`, `resync`.
//!
//! Resolution source is always the embedded catalog + `profile.toml` +
//! `[rules]` in `project.toml` — on-disk YAMLs are a readable cache, never
//! the source. Every subcommand honors the global `--json` flag via
//! [`OutputMode`]: JSON goes through the standard envelope, human output is
//! plain Markdown.

use std::path::Path;

use anyhow::{Context, Result};
use is_terminal::IsTerminal;
use serde::Serialize;

use super::context::{
    filter_for_stage, is_blocking, parse_stage, profile_summary, rank_for_task, render_rule_md,
    severity_counts, split_files_arg, truncate_to_budget, Stage,
};
use super::materialize::{materialize_all, resolve_for_project, write_project_init_rules};
use super::model::{load_catalog, ProfileAnswers, Rule, Severity};
use super::profile::{self};
use crate::core::output::OutputMode;

/// Active profile plus whether `profile.toml` existed (absent = transversal
/// only, 78 rules — never an error, never a block).
struct ActiveSet {
    answers: ProfileAnswers,
    present: bool,
    rules: Vec<Rule>,
}

fn load_active_set(project_dir: &Path) -> Result<ActiveSet> {
    let path = profile::profile_toml_path(project_dir);
    if path.is_file() {
        let (answers, _meta) = profile::load_profile(project_dir)?;
        let rules = resolve_for_project(project_dir, &answers)?;
        return Ok(ActiveSet {
            answers,
            present: true,
            rules,
        });
    }
    let answers = ProfileAnswers::default();
    let rules = resolve_for_project(project_dir, &answers)?;
    Ok(ActiveSet {
        answers,
        present: false,
        rules,
    })
}

fn parse_severity_filter(s: &str) -> Result<Severity> {
    match s.to_lowercase().as_str() {
        "must" => Ok(Severity::Must),
        "should" => Ok(Severity::Should),
        "avoid" => Ok(Severity::Avoid),
        _ => anyhow::bail!("unknown severity '{s}' (expected one of: must, should, avoid)"),
    }
}

/// `dectl rules list [--category X] [--severity S]`: browse the catalog.
pub fn run_list(
    category: Option<String>,
    severity: Option<String>,
    mode: OutputMode,
) -> Result<()> {
    let catalog = load_catalog()?;
    let wanted = severity.as_deref().map(parse_severity_filter).transpose()?;
    let rules: Vec<&Rule> = catalog
        .rules
        .iter()
        .filter(|r| category.as_ref().is_none_or(|c| r.section == *c))
        .filter(|r| wanted.is_none_or(|s| r.severity == s))
        .collect();

    #[derive(Serialize)]
    struct ListData<'a> {
        total: usize,
        category: Option<String>,
        severity: Option<String>,
        rules: Vec<&'a Rule>,
    }

    if mode.is_json() {
        mode.print(&ListData {
            total: rules.len(),
            category,
            severity,
            rules,
        })?;
        return Ok(());
    }
    println!("# Rules ({})", rules.len());
    println!();
    for rule in &rules {
        println!("{}", render_rule_md(rule));
    }
    Ok(())
}

/// `dectl rules search "<query>"`: case-insensitive match over id, section,
/// condition, action and tags.
pub fn run_search(query: String, mode: OutputMode) -> Result<()> {
    let catalog = load_catalog()?;
    let q = query.to_lowercase();
    let rules: Vec<&Rule> = catalog
        .rules
        .iter()
        .filter(|r| {
            r.id.to_lowercase().contains(&q)
                || r.section.to_lowercase().contains(&q)
                || r.condition.to_lowercase().contains(&q)
                || r.action.to_lowercase().contains(&q)
                || r.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect();

    #[derive(Serialize)]
    struct SearchData<'a> {
        query: String,
        total: usize,
        rules: Vec<&'a Rule>,
    }

    if mode.is_json() {
        mode.print(&SearchData {
            query,
            total: rules.len(),
            rules,
        })?;
        return Ok(());
    }
    println!("# Search: {query} ({})", rules.len());
    println!();
    if rules.is_empty() {
        println!("_no matches_");
        return Ok(());
    }
    for rule in &rules {
        println!("{}", render_rule_md(rule));
    }
    Ok(())
}

#[derive(Serialize)]
struct ResolveData {
    active_total: usize,
    must: usize,
    should: usize,
    avoid: usize,
    profile_present: bool,
    profile: ProfileAnswers,
    rules: Vec<Rule>,
}

/// `dectl rules resolve`: recompute and print the active ruleset.
pub fn run_resolve(mode: OutputMode) -> Result<()> {
    let project_dir = Path::new(".");
    let set = load_active_set(project_dir)?;
    let (must, should, avoid) = severity_counts(&set.rules);
    if mode.is_json() {
        mode.print(&ResolveData {
            active_total: set.rules.len(),
            must,
            should,
            avoid,
            profile_present: set.present,
            profile: set.answers,
            rules: set.rules,
        })?;
        return Ok(());
    }
    println!(
        "# Active ruleset ({} rules: MUST {must}/SHOULD {should}/AVOID {avoid})",
        set.rules.len()
    );
    if !set.present {
        println!("_no profile.toml: transversal rules only — run `dectl spec init`_");
    }
    println!();
    for rule in &set.rules {
        println!("{}", render_rule_md(rule));
    }
    Ok(())
}

#[derive(Serialize)]
struct ContextData {
    stage: String,
    budget: usize,
    used_tokens: usize,
    omitted: usize,
    profile_present: bool,
    active_flags: Vec<String>,
    active_total: usize,
    must: usize,
    should: usize,
    avoid: usize,
    blocking: usize,
    context: String,
    rule_ids: Vec<String>,
}

/// `dectl rules context --stage … [--files …] [--max-tokens N] [--task …]`.
#[allow(clippy::too_many_arguments)]
pub fn run_context(
    stage: String,
    files: Vec<String>,
    max_tokens: usize,
    task: Option<String>,
    mode: OutputMode,
) -> Result<()> {
    let stage = parse_stage(&stage)?;
    let project_dir = Path::new(".");
    let set = load_active_set(project_dir)?;
    let files = split_files_arg(&files);
    let task_text = task.unwrap_or_default();

    let (must, should, avoid) = severity_counts(&set.rules);
    let blocking = set.rules.iter().filter(|r| is_blocking(r)).count();
    let flags = set.answers.as_flag_map();
    let active_flags = profile_summary(&flags);

    // `spec` never carries rule text — metadata only (WHAT-vs-HOW).
    if stage == Stage::Spec {
        let context = render_spec_context(&set.answers, &set.rules, set.present);
        if mode.is_json() {
            mode.print(&ContextData {
                stage: "spec".to_string(),
                budget: max_tokens,
                used_tokens: super::context::count_tokens(&context),
                omitted: 0,
                profile_present: set.present,
                active_flags,
                active_total: set.rules.len(),
                must,
                should,
                avoid,
                blocking,
                context,
                rule_ids: Vec::new(),
            })?;
            return Ok(());
        }
        println!("{context}");
        return Ok(());
    }

    let filtered = filter_for_stage(&set.rules, stage);
    // Priority order per stage for truncation: plan keeps resolver order
    // (MUST first), task uses relevance rank, review keeps blocking first.
    let ordered: Vec<Rule> = match stage {
        Stage::Task => rank_for_task(&filtered, &task_text, &files)
            .into_iter()
            .map(|(r, _)| r)
            .collect(),
        Stage::Review => {
            let mut blocking_first: Vec<Rule> = filtered
                .iter()
                .filter(|r| is_blocking(r))
                .cloned()
                .collect();
            let mut avoid: Vec<Rule> = filtered
                .iter()
                .filter(|r| r.severity == Severity::Avoid)
                .cloned()
                .collect();
            avoid.sort_by(|a, b| a.id.cmp(&b.id));
            blocking_first.append(&mut avoid);
            blocking_first
        }
        _ => filtered,
    };

    let stage_name = match stage {
        Stage::Plan => "plan",
        Stage::Review => "review",
        _ => "task",
    };
    let header = format!(
        "# Rules context ({stage_name}: {} of {} active, budget {max_tokens} tokens)",
        ordered.len(),
        set.rules.len()
    );
    let bullets: Vec<String> = ordered.iter().map(render_rule_md).collect();
    let (kept, omitted) = truncate_to_budget(&bullets, max_tokens);
    let mut context = header;
    context.push_str("\n\n");
    context.push_str(&kept.join("\n"));
    if omitted > 0 {
        context.push_str(&format!("\n\n(... {omitted} rules omitted due to budget)"));
    }
    if !files.is_empty() {
        context.push_str(&format!("\n\n_files: {}_", files.join(", ")));
    }

    let used = super::context::count_tokens(&context);
    let rule_ids: Vec<String> = kept
        .iter()
        .filter_map(|b| b.strip_prefix("- **"))
        .filter_map(|rest| rest.split("**").next())
        .map(|s| s.to_string())
        .collect();

    if mode.is_json() {
        mode.print(&ContextData {
            stage: stage_name.to_string(),
            budget: max_tokens,
            used_tokens: used,
            omitted,
            profile_present: set.present,
            active_flags,
            active_total: set.rules.len(),
            must,
            should,
            avoid,
            blocking,
            context,
            rule_ids,
        })?;
        return Ok(());
    }
    println!("{context}");
    Ok(())
}

fn render_spec_context(answers: &ProfileAnswers, active: &[Rule], present: bool) -> String {
    let (must, should, avoid) = severity_counts(active);
    let flags = answers.as_flag_map();
    let mut out = format!(
        "# Rules context (spec: {} active — MUST {must}/SHOULD {should}/AVOID {avoid})",
        active.len()
    );
    out.push_str("\n\n_profile: ");
    let mut names: Vec<&str> = super::model::KNOWN_FLAGS.to_vec();
    names.sort();
    let parts: Vec<String> = names
        .iter()
        .map(|f| format!("{f}={}", flags.get(*f).unwrap_or(&false)))
        .collect();
    out.push_str(&parts.join(" "));
    out.push('_');
    if !present {
        out.push_str("\n_no profile.toml: transversal rules only_");
    }
    out.push_str(
        "\n\n_Rules not included (WHAT-vs-HOW): spec receives metadata, never rule text._",
    );
    out
}

/// `dectl rules profile update`: re-run the questionnaire, persist with
/// `answered_via = "profile_update"`, re-resolve and regenerate everything.
pub fn run_profile_update(non_interactive: bool, mode: OutputMode) -> Result<()> {
    let project_dir = Path::new(".");
    if non_interactive || !std::io::stdin().is_terminal() {
        anyhow::bail!(
            "profile update needs an interactive terminal (or pipe answers on stdin); \
             alternatively edit .dec/rules/profile.toml by hand"
        );
    }
    let current = profile::profile_toml_path(project_dir).is_file();
    eprintln!(
        "Re-answering the profile questionnaire (all 8 flags, current profile kept as reference)."
    );
    if current {
        let (answers, _) = profile::load_profile(project_dir)
            .context("current .dec/rules/profile.toml is invalid — fix it before updating")?;
        eprintln!("{}", profile::render_profile_block(&answers));
    }
    let stdin = std::io::stdin();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    let answers =
        profile::run_questionnaire_with_io(&mut reader, &mut writer)?.ok_or_else(|| {
            anyhow::anyhow!("questionnaire aborted (EOF on stdin): profile unchanged")
        })?;
    drop(writer);
    profile::save_profile(project_dir, &answers, "profile_update")?;
    let report = materialize_all(project_dir, &answers)?;

    #[derive(Serialize)]
    struct UpdateData {
        answered_via: String,
        active_total: usize,
        must: usize,
        should: usize,
        avoid: usize,
        written: Vec<String>,
        removed: Vec<String>,
    }

    if mode.is_json() {
        mode.print(&UpdateData {
            answered_via: "profile_update".to_string(),
            active_total: report.active_total,
            must: report.severity_counts.0,
            should: report.severity_counts.1,
            avoid: report.severity_counts.2,
            written: report.written,
            removed: report.removed,
        })?;
        return Ok(());
    }
    println!(
        "Profile updated (profile_update): {} active rules (MUST {}/SHOULD {}/AVOID {}); \
         see .dec/rules/active-ruleset.md",
        report.active_total,
        report.severity_counts.0,
        report.severity_counts.1,
        report.severity_counts.2,
    );
    Ok(())
}

/// `dectl rules resync`: regenerate every on-disk YAML from the embedded
/// catalog, overwriting manual edits (headers say DO NOT EDIT).
pub fn run_resync(mode: OutputMode) -> Result<()> {
    let project_dir = Path::new(".");
    eprintln!("Warning: resync overwrites manual edits under .dec/rules/ (generated files are read-only cache).");
    let wrote_init_only: bool;
    let (active_total, counts, written, skipped, removed);
    if profile::profile_toml_path(project_dir).is_file() {
        let (answers, _) = profile::load_profile(project_dir)?;
        let report = materialize_all(project_dir, &answers)?;
        active_total = report.active_total;
        counts = report.severity_counts;
        written = report.written;
        skipped = report.skipped_unchanged;
        removed = report.removed;
        wrote_init_only = false;
    } else {
        write_project_init_rules(project_dir)?;
        let set = load_active_set(project_dir)?;
        active_total = set.rules.len();
        counts = severity_counts(&set.rules);
        written = vec![
            ".dec/rules/transversal.yaml".to_string(),
            ".dec/rules/profile.schema.yaml".to_string(),
        ];
        skipped = Vec::new();
        removed = Vec::new();
        wrote_init_only = true;
    }

    #[derive(Serialize)]
    struct ResyncData {
        active_total: usize,
        must: usize,
        should: usize,
        avoid: usize,
        init_only: bool,
        written: Vec<String>,
        skipped_unchanged: Vec<String>,
        removed: Vec<String>,
    }

    if mode.is_json() {
        mode.print(&ResyncData {
            active_total,
            must: counts.0,
            should: counts.1,
            avoid: counts.2,
            init_only: wrote_init_only,
            written,
            skipped_unchanged: skipped,
            removed,
        })?;
        return Ok(());
    }
    if wrote_init_only {
        println!(
            "Resynced from embedded catalog (no profile.toml: transversal + schema only): \
             {active_total} rules; run `dectl spec init` for the full mirror."
        );
    } else {
        println!(
            "Resynced from embedded catalog: {active_total} active rules \
             (MUST {}/SHOULD {}/AVOID {}); {} written, {} unchanged, {} removed.",
            counts.0,
            counts.1,
            counts.2,
            written.len(),
            skipped.len(),
            removed.len(),
        );
    }
    Ok(())
}
