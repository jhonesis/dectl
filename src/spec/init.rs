use crate::bail_app_err;
use anyhow::{Context, Result};
use std::path::Path;

use super::bridge;
use super::templates;
use crate::rules::{materialize, profile};

pub fn run(json: bool, non_interactive: bool, from: Option<&Path>) -> Result<()> {
    let project_dir = Path::new(".");

    let dec_exists = project_dir.join(".dec").exists();
    if !dec_exists {
        bail_app_err!(
            ".dec/ not found. Run `dectl project init` first.",
            "Run `dectl project init` from a dectl project directory"
        );
    }

    templates::ensure_sdd_dir(project_dir)?;

    bridge::update_project_toml(project_dir)?;
    bridge::update_project_isa(project_dir)?;

    let from_content = if let Some(from_path) = from {
        if !from_path.exists() {
            bail_app_err!(
                format!("File not found: {}", from_path.display()),
                format!("The file '{}' does not exist", from_path.display())
            );
        }
        let content = std::fs::read_to_string(from_path)
            .with_context(|| format!("Failed to read file: {}", from_path.display()))?;
        if content.trim().is_empty() {
            bail_app_err!(
                format!("File is empty: {}", from_path.display()),
                format!("The file '{}' is empty", from_path.display())
            );
        }
        Some(content)
    } else {
        None
    };

    // Rules profile, once-only (REQ-rules-002): ask the 8 flags on stdin the
    // first time, skip without TTY (warning, rules pending), never re-ask.
    // Then resolve the active set and mirror it to disk (REQ-rules-005).
    let mut profile_block: Option<String> = None;
    let mut rules_summary: Option<String> = None;
    let mut profile_pending_warning: Option<String> = None;
    let mut asked_now = false;
    match profile::ensure_profile(project_dir, non_interactive, "spec_init")? {
        profile::ProfileOutcome::Existing { answers } => {
            let report = materialize::materialize_all(project_dir, &answers)?;
            profile_block = Some(profile::render_profile_block(&answers));
            rules_summary = Some(format!(
                ".dec/rules/ active: {} rules (MUST {}/SHOULD {}/AVOID {}); \
                 see .dec/rules/active-ruleset.md",
                report.active_total,
                report.severity_counts.0,
                report.severity_counts.1,
                report.severity_counts.2,
            ));
        }
        profile::ProfileOutcome::Created { answers } => {
            asked_now = true;
            let report = materialize::materialize_all(project_dir, &answers)?;
            profile_block = Some(profile::render_profile_block(&answers));
            rules_summary = Some(format!(
                ".dec/rules/ active: {} rules (MUST {}/SHOULD {}/AVOID {}); \
                 see .dec/rules/active-ruleset.md",
                report.active_total,
                report.severity_counts.0,
                report.severity_counts.1,
                report.severity_counts.2,
            ));
        }
        profile::ProfileOutcome::Skipped { reason } => {
            eprintln!("Warning: {reason}");
            profile_pending_warning = Some(reason);
        }
    }

    let mut message = r#".dec/sdd/ ready
.dec/config/project.toml updated
.dec/isa/project.isa.md updated
Agent: interview the user and create specs/ with real content
  - Read .dec/sdd/references/templates.md for document templates
  - Read .dec/sdd/SKILL.md for the SDD workflow
  - Create specs/ in the project root"#
        .to_string();

    if let Some(ref block) = profile_block {
        message.push_str(&format!("\n\n{block}"));
        if let Some(ref summary) = rules_summary {
            message.push_str(&format!("\n{summary}"));
        }
        if asked_now {
            message.push_str("\nProfile answered now and saved to .dec/rules/profile.toml.");
        }
    } else if let Some(ref reason) = profile_pending_warning {
        message.push_str(&format!("\n\nRules pending: {reason}"));
    }

    if let Some(ref content) = from_content {
        message.push_str(&format!(
            "\n\n[SOURCE FILE CONTENT]\n{}\n[/SOURCE FILE CONTENT]\nUse the above content as input to create specs following SKILL.md rules.",
            content
        ));
    }

    if json {
        let envelope = serde_json::json!({
            "status": "ok",
            "data": {
                "message": ".dec/sdd/ ready",
                "bridge": {
                    "project_toml": true,
                    "project_isa": true
                },
                "profile": profile_block.as_ref().map(|b| {
                    serde_json::json!({
                        "answered_now": asked_now,
                        "block": b,
                    })
                }),
                "rules": rules_summary.as_ref().map(|s| {
                    serde_json::json!({ "summary": s })
                }),
                "profile_pending": profile_pending_warning,
                "next": "Interview the user and create specs/ with SDD documents",
                "from_file": from.map(|p| p.display().to_string())
            }
        });
        println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
    } else {
        println!("{}", message);
    }

    Ok(())
}
