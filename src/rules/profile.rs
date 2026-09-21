//! Profile questionnaire for `spec init` (REQ-rules-002).
//!
//! - [`ensure_profile`]: once-only flow — if `.dec/rules/profile.toml`
//!   exists it is loaded and returned without asking; otherwise the 8
//!   questions from the embedded `PROFILE_SCHEMA_YAML` are rendered
//!   sequentially on stdin (line-by-line, `memory/delete.rs` precedent).
//! - Without an interactive terminal (or with `--non-interactive`) the
//!   questionnaire is skipped with a warning: no `profile.toml` is created
//!   and the rules stay pending (`spec init` still succeeds).
//! - Skipped `depends_on` questions persist as explicit `false` — the
//!   resolver never distinguishes "absent" from `false`.
//! - [`render_profile_block`]: the `[PROFILE]` agent block (same pattern as
//!   `[SOURCE FILE CONTENT]`): 8 answers + `tier_hint` + the no-re-ask rule.

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use is_terminal::IsTerminal;
use serde::Deserialize;

use super::data::PROFILE_SCHEMA_YAML;
use super::model::{ProfileAnswers, ProfileMeta, KNOWN_FLAGS};

/// Flags that own a materialized `profiles/{flag}.yaml` when active.
/// `handles_pii`/`long_lived` are meta-flags: persisted, never materialized.
pub const CONDITIONAL_FLAGS: [&str; 6] = [
    "has_sql",
    "public_api",
    "has_web_ui",
    "has_auth",
    "realtime",
    "cpu_intensive",
];

/// Outcome of [`ensure_profile`].
pub enum ProfileOutcome {
    /// `profile.toml` already existed — loaded without asking.
    Existing { answers: ProfileAnswers },
    /// Questionnaire answered now and persisted.
    Created { answers: ProfileAnswers },
    /// No TTY / `--non-interactive`: warned and continued without a profile.
    Skipped { reason: String },
}

/// Path to `.dec/rules/profile.toml` for a project directory.
pub fn profile_toml_path(project_dir: &Path) -> PathBuf {
    project_dir.join(".dec/rules/profile.toml")
}

#[derive(Debug, Deserialize)]
struct Schema {
    #[serde(default)]
    questions: Vec<SchemaQuestion>,
}

#[derive(Debug, Deserialize)]
struct SchemaQuestion {
    #[allow(dead_code)]
    id: Option<String>,
    flag: String,
    #[serde(rename = "type", default)]
    _qtype: Option<String>,
    #[serde(default)]
    order: u32,
    prompt: String,
    #[serde(default)]
    help: Option<String>,
    #[serde(default)]
    default: Option<bool>,
    #[serde(default)]
    depends_on: Option<DependsOn>,
}

#[derive(Debug, Deserialize, Clone)]
struct DependsOn {
    flag: String,
    equals: bool,
}

/// Load an existing profile (validates unknown flags with an error naming
/// the flag — never silently ignored).
pub fn load_profile(project_dir: &Path) -> Result<(ProfileAnswers, ProfileMeta)> {
    let path = profile_toml_path(project_dir);
    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    ProfileAnswers::parse_profile_toml(&content)
}

/// Persist the 8 flags plus `[profile.meta]` (`answered_at` ISO-8601,
/// `answered_via`, `schema_version = 2`).
pub fn save_profile(project_dir: &Path, answers: &ProfileAnswers, via: &str) -> Result<()> {
    let path = profile_toml_path(project_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    let now = chrono::Utc::now().to_rfc3339();
    let mut text = String::from("# .dec/rules/profile.toml - schema_version 2\n");
    text.push_str("# One-time questionnaire answers from `dectl spec init`. DO NOT EDIT BY HAND\n");
    text.push_str("# (use `dectl rules profile update` to change scope).\n\n");
    text.push_str("[profile]\n");
    for flag in KNOWN_FLAGS {
        let value = match flag {
            "handles_pii" => answers.handles_pii,
            "has_sql" => answers.has_sql,
            "public_api" => answers.public_api,
            "has_web_ui" => answers.has_web_ui,
            "has_auth" => answers.has_auth,
            "realtime" => answers.realtime,
            "cpu_intensive" => answers.cpu_intensive,
            "long_lived" => answers.long_lived,
            _ => false,
        };
        text.push_str(&format!("{flag} = {value}\n"));
    }
    text.push_str("\n[profile.meta]\n");
    text.push_str(&format!("answered_at = \"{now}\"\n"));
    text.push_str(&format!("answered_via = \"{via}\"\n"));
    text.push_str("schema_version = 2\n");
    fs::write(&path, &text).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Tier hint derived from the profile (`handles_pii` → CRITICAL).
/// Advisory only: SDD Step 0 still decides the tier; conflicts resolve
/// toward CRITICAL (safe bias).
pub fn tier_hint(answers: &ProfileAnswers) -> &'static str {
    if answers.handles_pii {
        "CRITICAL"
    } else {
        "STANDARD"
    }
}

/// Render the `[PROFILE]` agent block: 8 answers + `tier_hint` + the
/// no-re-ask rule (anti-double-questioning with the SDD intake).
pub fn render_profile_block(answers: &ProfileAnswers) -> String {
    let mut out = String::from("[PROFILE]\n");
    for flag in KNOWN_FLAGS {
        let value = match flag {
            "handles_pii" => answers.handles_pii,
            "has_sql" => answers.has_sql,
            "public_api" => answers.public_api,
            "has_web_ui" => answers.has_web_ui,
            "has_auth" => answers.has_auth,
            "realtime" => answers.realtime,
            "cpu_intensive" => answers.cpu_intensive,
            "long_lived" => answers.long_lived,
            _ => false,
        };
        out.push_str(&format!("{flag} = {value}\n"));
    }
    out.push_str(&format!("tier_hint = {}\n", tier_hint(answers)));
    out.push_str("[/PROFILE]\n");
    out.push_str("Instruction: Do NOT re-ask these flags - ");
    out.push_str("profile.toml is the interview input. ");
    out.push_str("If --from content contradicts a flag, point out the contradiction ");
    out.push_str("and propose `dectl rules profile update`; never overwrite it silently.");
    out
}

/// Parse the embedded questionnaire bank, ordered by `order`.
fn ordered_questions() -> Result<Vec<SchemaQuestion>> {
    let schema: Schema = serde_yaml::from_str(PROFILE_SCHEMA_YAML)
        .context("embedded profile.schema.yaml failed to parse")?;
    let mut questions = schema.questions;
    questions.sort_by_key(|q| q.order);
    Ok(questions)
}

fn parse_yes_no(line: &str, default: bool) -> Option<bool> {
    let t = line.trim().to_lowercase();
    if t.is_empty() {
        return Some(default);
    }
    match t.as_str() {
        "y" | "yes" | "s" | "si" | "sí" | "true" | "1" | "t" => Some(true),
        "n" | "no" | "false" | "0" | "f" => Some(false),
        _ => None,
    }
}

fn default_for(q: &SchemaQuestion) -> bool {
    q.default.unwrap_or(false)
}

fn short_default(default: bool) -> &'static str {
    if default {
        "Y/n"
    } else {
        "y/N"
    }
}

/// Run the questionnaire against injected I/O (testable without a TTY).
///
/// Returns `Ok(None)` on EOF (piped stdin closed): the caller treats it as
/// a skip, never blocking. `depends_on` misses persist as explicit `false`.
pub fn run_questionnaire_with_io<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<Option<ProfileAnswers>> {
    let questions = ordered_questions()?;
    let mut flags: HashMap<String, bool> = HashMap::new();
    for q in &questions {
        if let Some(dep) = &q.depends_on {
            if *flags.get(&dep.flag).unwrap_or(&false) != dep.equals {
                flags.insert(q.flag.clone(), false);
                continue;
            }
        }
        let default = default_for(q);
        if let Some(help) = &q.help {
            writeln!(writer, "{} [{}]", q.prompt, short_default(default))?;
            writeln!(writer, "  ({})", help.trim().replace('\n', " "))?;
        } else {
            write!(writer, "{} [{}]: ", q.prompt, short_default(default))?;
        }
        writer.flush()?;
        loop {
            let mut line = String::new();
            let n = reader
                .read_line(&mut line)
                .context("Failed to read questionnaire answer")?;
            if n == 0 {
                return Ok(None);
            }
            match parse_yes_no(&line, default) {
                Some(v) => {
                    flags.insert(q.flag.clone(), v);
                    break;
                }
                None => {
                    write!(writer, "Please answer y/n [{}]: ", short_default(default))?;
                    writer.flush()?;
                }
            }
        }
    }
    Ok(Some(ProfileAnswers {
        handles_pii: *flags.get("handles_pii").unwrap_or(&false),
        has_sql: *flags.get("has_sql").unwrap_or(&false),
        public_api: *flags.get("public_api").unwrap_or(&false),
        has_web_ui: *flags.get("has_web_ui").unwrap_or(&false),
        has_auth: *flags.get("has_auth").unwrap_or(&false),
        realtime: *flags.get("realtime").unwrap_or(&false),
        cpu_intensive: *flags.get("cpu_intensive").unwrap_or(&false),
        long_lived: *flags.get("long_lived").unwrap_or(&false),
    }))
}

/// Once-only entry point used by `spec init`.
///
/// - Profile exists → load, no questions.
/// - `non_interactive` or stdin without TTY → skip with a warning, `Ok(None)`.
/// - Otherwise ask on stdin/stdout and persist with `via`.
pub fn ensure_profile(
    project_dir: &Path,
    non_interactive: bool,
    via: &str,
) -> Result<ProfileOutcome> {
    let path = profile_toml_path(project_dir);
    if path.exists() {
        let (answers, _meta) = load_profile(project_dir)?;
        return Ok(ProfileOutcome::Existing { answers });
    }
    if non_interactive || !std::io::stdin().is_terminal() {
        return Ok(ProfileOutcome::Skipped {
            reason: "no interactive terminal and no .dec/rules/profile.toml: \
                     skipping questionnaire, rules pending \
                     (run `dectl rules profile update` later)"
                .to_string(),
        });
    }
    let stdin = std::io::stdin();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    match run_questionnaire_with_io(&mut reader, &mut writer)? {
        Some(answers) => {
            drop(writer);
            save_profile(project_dir, &answers, via)?;
            Ok(ProfileOutcome::Created { answers })
        }
        None => Ok(ProfileOutcome::Skipped {
            reason: "questionnaire aborted (EOF on stdin): rules pending \
                     (run `dectl rules profile update` later)"
                .to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn answers_from(lines: &str) -> Option<ProfileAnswers> {
        let mut reader = std::io::BufReader::new(Cursor::new(lines));
        let mut writer = Vec::new();
        run_questionnaire_with_io(&mut reader, &mut writer).expect("questionnaire must not fail")
    }

    #[test]
    fn all_yes_asks_8_questions_in_order() {
        // q6 (realtime) only appears because public_api=true above it.
        let answers = answers_from("y\ny\ny\ny\ny\ny\ny\ny\n").expect("must answer");
        assert!(answers.handles_pii);
        assert!(answers.has_sql);
        assert!(answers.public_api);
        assert!(answers.has_web_ui);
        assert!(answers.has_auth);
        assert!(answers.realtime);
        assert!(answers.cpu_intensive);
        assert!(answers.long_lived);
    }

    #[test]
    fn depends_on_skip_persists_realtime_false() {
        // public_api=n → realtime (depends_on public_api=true) skipped as false.
        let answers = answers_from("n\nn\nn\nn\nn\nn\nn\n").expect("must answer");
        assert!(!answers.public_api);
        assert!(
            !answers.realtime,
            "skipped depends_on must persist as false"
        );
    }

    #[test]
    fn empty_lines_take_schema_defaults() {
        // q8 defaults to true, the rest to false.
        let answers = answers_from("\n\n\n\n\n\n\n\n").expect("must answer");
        assert!(!answers.handles_pii);
        assert!(!answers.has_sql);
        assert!(answers.long_lived, "q8 default is true");
    }

    #[test]
    fn eof_returns_none_without_blocking() {
        let answers = answers_from("");
        assert!(answers.is_none(), "closed stdin must skip, not block");
    }

    #[test]
    fn invalid_answer_reprompts_and_accepts_retry() {
        let mut reader = std::io::BufReader::new(Cursor::new("maybe\ny\nn\nn\nn\nn\nn\nn\nn\n"));
        let mut writer = Vec::new();
        let answers = run_questionnaire_with_io(&mut reader, &mut writer)
            .expect("must not fail")
            .expect("must answer");
        assert!(answers.handles_pii);
        let prompt = String::from_utf8(writer).unwrap();
        assert!(prompt.contains("Please answer y/n"));
    }

    #[test]
    fn tier_hint_follows_handles_pii() {
        let mut answers = ProfileAnswers::default();
        assert_eq!(tier_hint(&answers), "STANDARD");
        answers.handles_pii = true;
        assert_eq!(tier_hint(&answers), "CRITICAL");
    }

    #[test]
    fn profile_block_has_no_reask_instruction() {
        let answers = ProfileAnswers::default();
        let block = render_profile_block(&answers);
        assert!(block.contains("[PROFILE]"));
        assert!(block.contains("[/PROFILE]"));
        assert!(block.contains("tier_hint = STANDARD"));
        assert!(
            block.contains("Do NOT re-ask"),
            "agent must not re-ask answered flags"
        );
        assert!(block.contains("dectl rules profile update"));
        for flag in KNOWN_FLAGS {
            assert!(block.contains(flag), "block must list {flag}");
        }
    }

    #[test]
    fn save_and_load_roundtrip_preserves_flags_and_meta() {
        let tmp = tempfile::tempdir().unwrap();
        let answers = ProfileAnswers {
            handles_pii: true,
            has_sql: true,
            ..Default::default()
        };
        save_profile(tmp.path(), &answers, "spec_init").unwrap();
        let (loaded, meta) = load_profile(tmp.path()).unwrap();
        assert!(loaded.handles_pii && loaded.has_sql);
        assert!(!loaded.public_api);
        assert_eq!(meta.answered_via.as_deref(), Some("spec_init"));
        assert_eq!(meta.schema_version, Some(2));
        assert!(meta.answered_at.is_some());
    }

    #[test]
    fn profile_strings_carry_no_diacritics() {
        // Anti-regression (REQ-007): the [PROFILE] agent block and the
        // persisted profile.toml header must stay English/ASCII.
        let block = render_profile_block(&ProfileAnswers::default());
        assert!(
            block.is_ascii(),
            "profile block must be ASCII-only, got:\n{block}"
        );
        let tmp = tempfile::tempdir().unwrap();
        save_profile(tmp.path(), &ProfileAnswers::default(), "spec_init").unwrap();
        let text = std::fs::read_to_string(profile_toml_path(tmp.path())).unwrap();
        assert!(
            text.is_ascii(),
            "profile.toml must be ASCII-only, got:\n{text}"
        );
    }
}
