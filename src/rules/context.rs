//! Stage-aware context for the rules subsystem (REQ-rules-003, REQ-rules-006).
//!
//! - [`Stage`]: the four consumers (`spec` → metadata only, never rule text;
//!   `plan` → MUST + SHOULD; `task` → MUST → AVOID → SHOULD ranked by
//!   relevance; `review` → blocking set (§7.1) + AVOID).
//! - [`score_rule`] / [`rank_for_task`]: token-overlap relevance between
//!   (rule tags + `condition`/`action` words) and (task description + `--files`
//!   path components). Deterministic order `(severity group, score desc, id)`.
//! - [`split_files_arg`]: `--files` tolerates spaces, commas and newlines
//!   (the reviewer feeds `git diff --name-only` output via interpolation).
//! - [`truncate_to_budget`]: `words × 1.3` estimator (same as
//!   `project::context`), MUST/AVOID first, footer with omitted count —
//!   never a silent cut.

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};

use super::model::{Rule, Severity};

/// Default token budget for `rules context` (the researcher already spends
/// 4000 on project context; the rules slice must be smaller).
pub const DEFAULT_BUDGET: usize = 2000;

/// Sections whose MUST rules form the blocking gate set (§7.1: security and
/// correctness only — design advice warns, never fails).
pub const BLOCKING_SECTIONS: [&str; 3] = ["Security", "Secrets Management", "Error Handling"];

/// Context consumer stage (`--stage`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Spec,
    Plan,
    Task,
    Review,
}

/// Parse `--stage` (case-insensitive). Unknown values are an error naming
/// the value (never silently defaulted — the wrong stage leaks HOW into
/// `spec.md` or starves the gate).
pub fn parse_stage(s: &str) -> Result<Stage> {
    match s.to_lowercase().as_str() {
        "spec" => Ok(Stage::Spec),
        "plan" => Ok(Stage::Plan),
        "task" => Ok(Stage::Task),
        "review" => Ok(Stage::Review),
        _ => Err(anyhow!(
            "unknown stage '{s}' (expected one of: spec, plan, task, review)"
        )),
    }
}

/// Whether a rule belongs to the blocking gate set: MUST in a §7.1 section.
/// Evaluated over the already-resolved active set, so `[rules] disabled` /
/// `severity_override` can move rules in or out without data changes.
pub fn is_blocking(rule: &Rule) -> bool {
    rule.severity == Severity::Must && BLOCKING_SECTIONS.contains(&rule.section.as_str())
}

/// Severity rank for stage ordering. `task`/`review` rank AVOID above SHOULD:
/// a confirmed antipattern blocks, design advice only warns.
fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Must => 0,
        Severity::Avoid => 1,
        Severity::Should => 2,
    }
}

/// Filter the resolved active set for a stage (before ranking/truncation).
///
/// - `spec` returns an empty vec: the caller renders metadata only, never
///   rule text (WHAT-vs-HOW gate).
/// - `plan` returns MUST + SHOULD (architecture decisions).
/// - `task` returns everything (ranking decides the order).
/// - `review` returns the blocking MUST set + every AVOID (small, decidable).
pub fn filter_for_stage(active: &[Rule], stage: Stage) -> Vec<Rule> {
    match stage {
        Stage::Spec => Vec::new(),
        Stage::Plan => active
            .iter()
            .filter(|r| r.severity != Severity::Avoid)
            .cloned()
            .collect(),
        Stage::Task => active.to_vec(),
        Stage::Review => active
            .iter()
            .filter(|r| is_blocking(r) || r.severity == Severity::Avoid)
            .cloned()
            .collect(),
    }
}

/// Split `--files` values on whitespace (spaces, newlines), commas and
/// semicolons; trims, drops empties, dedups preserving first-seen order.
pub fn split_files_arg(files: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for entry in files {
        for part in entry.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
            let part = part.trim();
            if !part.is_empty() && seen.insert(part.to_string()) {
                out.push(part.to_string());
            }
        }
    }
    out
}

/// Lowercase alphanumeric tokens (`len >= 3` to drop noise like `de`, `la`).
fn tokenize(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter_map(|w| {
            let w = w.to_lowercase();
            if w.chars().count() >= 3 {
                Some(w)
            } else {
                None
            }
        })
        .collect()
}

/// Path components of `--files` as tokens (`src/rules/mod.rs` → `src`,
/// `rules`, `mod`, `rs`).
fn file_tokens(files: &[String]) -> HashSet<String> {
    let mut out = HashSet::new();
    for f in files {
        for part in f.split(|c: char| !c.is_alphanumeric()) {
            let part = part.to_lowercase();
            if part.chars().count() >= 3 {
                out.insert(part);
            }
        }
    }
    out
}

/// Relevance score: tag hits weigh double (curated keywords), `condition` /
/// `action` word hits weigh single.
pub fn score_rule(rule: &Rule, query: &HashSet<String>, files: &HashSet<String>) -> usize {
    let mut score = 0;
    for tag in &rule.tags {
        let t = tag.to_lowercase();
        if query.contains(&t) {
            score += 2;
        }
        if files.contains(&t) {
            score += 2;
        }
    }
    let body = tokenize(&format!("{} {}", rule.condition, rule.action));
    score += body.intersection(query).count();
    score += body.intersection(files).count();
    score
}

/// Rank rules for the `task` stage: severity group (MUST → AVOID → SHOULD),
/// then score descending, then ID ascending (fully deterministic).
pub fn rank_for_task(rules: &[Rule], task_text: &str, files: &[String]) -> Vec<(Rule, usize)> {
    let query = tokenize(task_text);
    let ftokens = file_tokens(files);
    let mut scored: Vec<(Rule, usize)> = rules
        .iter()
        .map(|r| (r.clone(), score_rule(r, &query, &ftokens)))
        .collect();
    scored.sort_by(|a, b| {
        severity_rank(a.0.severity)
            .cmp(&severity_rank(b.0.severity))
            .then(b.1.cmp(&a.1))
            .then(a.0.id.cmp(&b.0.id))
    });
    scored
}

/// Token estimator shared with `project context`: words × 1.3.
pub fn count_tokens(text: &str) -> usize {
    let words = text.split_whitespace().count();
    ((words as f64) * 1.3) as usize
}

/// One rule rendered as a Markdown bullet (the unit the budget truncates).
pub fn render_rule_md(rule: &Rule) -> String {
    let sev = match rule.severity {
        Severity::Must => "MUST",
        Severity::Should => "SHOULD",
        Severity::Avoid => "AVOID",
    };
    format!(
        "- **{}** ({}, {}): {} → {}",
        rule.id, rule.section, sev, rule.condition, rule.action
    )
}

/// Take priority-ordered bullets while they fit in `budget` tokens.
/// Returns the kept bullets plus the omitted count (0 = nothing cut).
pub fn truncate_to_budget(blocks: &[String], budget: usize) -> (Vec<String>, usize) {
    let mut kept = Vec::new();
    let mut used = 0;
    for block in blocks {
        let cost = count_tokens(block);
        if used + cost > budget && !kept.is_empty() {
            break;
        }
        // Always keep the first block even if it alone exceeds the budget:
        // an empty context with no footer would be a silent cut.
        used += cost;
        kept.push(block.clone());
        if used > budget {
            break;
        }
    }
    let omitted = blocks.len() - kept.len();
    (kept, omitted)
}

/// Severity counts over a rule slice: (MUST, SHOULD, AVOID).
pub fn severity_counts(rules: &[Rule]) -> (usize, usize, usize) {
    (
        rules
            .iter()
            .filter(|r| r.severity == Severity::Must)
            .count(),
        rules
            .iter()
            .filter(|r| r.severity == Severity::Should)
            .count(),
        rules
            .iter()
            .filter(|r| r.severity == Severity::Avoid)
            .count(),
    )
}

/// Human-readable profile inventory for `spec`-stage metadata.
pub fn profile_summary(active_flags: &HashMap<String, bool>) -> Vec<String> {
    let mut flags: Vec<String> = active_flags
        .iter()
        .filter(|(_, v)| **v)
        .map(|(k, _)| k.clone())
        .collect();
    flags.sort();
    flags
}

#[cfg(test)]
mod tests {
    use super::super::model::load_catalog;
    use super::*;

    fn catalog() -> Vec<Rule> {
        load_catalog().unwrap().rules
    }

    #[test]
    fn parse_stage_accepts_known_and_rejects_unknown() {
        assert_eq!(parse_stage("spec").unwrap(), Stage::Spec);
        assert_eq!(parse_stage("PLAN").unwrap(), Stage::Plan);
        assert_eq!(parse_stage("Task").unwrap(), Stage::Task);
        assert_eq!(parse_stage("review").unwrap(), Stage::Review);
        let err = parse_stage("design").unwrap_err().to_string();
        assert!(err.contains("design"), "error must name the value: {err}");
    }

    #[test]
    fn spec_stage_returns_no_rules() {
        let active = catalog();
        assert!(filter_for_stage(&active, Stage::Spec).is_empty());
    }

    #[test]
    fn plan_stage_excludes_avoid() {
        let active = catalog();
        let plan = filter_for_stage(&active, Stage::Plan);
        assert!(!plan.is_empty());
        assert!(plan.iter().all(|r| r.severity != Severity::Avoid));
    }

    #[test]
    fn review_stage_is_blocking_must_plus_avoid_only() {
        let active = catalog();
        let review = filter_for_stage(&active, Stage::Review);
        assert!(!review.is_empty());
        assert!(review
            .iter()
            .all(|r| is_blocking(r) || r.severity == Severity::Avoid));
        assert!(review.iter().any(|r| r.severity == Severity::Avoid));
        // Design-advice MUST (e.g. 14.x Sorting) warns, never blocks.
        assert!(review
            .iter()
            .all(|r| r.severity != Severity::Must || is_blocking(r)));
    }

    #[test]
    fn blocking_set_matches_section_names() {
        let active = catalog();
        let blocking: Vec<&Rule> = active.iter().filter(|r| is_blocking(r)).collect();
        assert!(!blocking.is_empty());
        assert!(blocking.iter().any(|r| r.id == "13.1"));
        assert!(blocking
            .iter()
            .all(|r| BLOCKING_SECTIONS.contains(&r.section.as_str())));
    }

    #[test]
    fn split_files_accepts_newlines_commas_and_repeats() {
        let files = vec![
            "src/a.rs src/b.rs,src/c.rs".to_string(),
            "src/a.rs\nsrc/d.rs".to_string(),
        ];
        assert_eq!(
            split_files_arg(&files),
            vec!["src/a.rs", "src/b.rs", "src/c.rs", "src/d.rs"]
        );
        assert!(split_files_arg(&["  ,\n ".to_string()]).is_empty());
    }

    #[test]
    fn rank_for_task_is_deterministic_and_severity_grouped() {
        let active = catalog();
        let files = vec!["src/db.rs".to_string()];
        let first = rank_for_task(&active, "SQL injection query", &files);
        let second = rank_for_task(&active, "SQL injection query", &files);
        let ids_first: Vec<&str> = first.iter().map(|(r, _)| r.id.as_str()).collect();
        let ids_second: Vec<&str> = second.iter().map(|(r, _)| r.id.as_str()).collect();
        assert_eq!(ids_first, ids_second);
        let rank = |r: &Rule| severity_rank(r.severity);
        assert!(first.windows(2).all(|w| rank(&w[0].0) <= rank(&w[1].0)));
        // SQL-flavored query ranks a has_sql rule above an unrelated one.
        let pos_13_1 = ids_first.iter().position(|id| *id == "13.1").unwrap();
        let pos_14_5 = ids_first.iter().position(|id| *id == "14.5").unwrap();
        assert!(pos_13_1 < pos_14_5);
    }

    #[test]
    fn truncate_keeps_priority_order_and_reports_omitted() {
        let blocks: Vec<String> = (0..10).map(|i| format!("rule bullet number {i}")).collect();
        let (kept, omitted) = truncate_to_budget(&blocks, 6);
        assert!(!kept.is_empty());
        assert_eq!(kept.len() + omitted, blocks.len());
        assert!(kept[0].contains('0'), "priority order preserved");
        let (all, none) = truncate_to_budget(&blocks, 1_000_000);
        assert_eq!(all.len(), blocks.len());
        assert_eq!(none, 0);
    }

    #[test]
    fn count_tokens_matches_project_context_estimator() {
        assert_eq!(count_tokens("one two three four"), (4.0 * 1.3) as usize);
    }
}
