# Changelog — dectl

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## [1.1.0] — 2026-09-21 — Development rules subsystem (`rules/`) + full English

First public release of the rules subsystem (REQ-rules-001…REQ-rules-008,
REQ-007): 106-rule catalog embedded in the binary, per-project profiles via
one-time questionnaire, budgeted stage-aware context, and a blocking reviewer
gate (security/correctness only) with static pre-triage. All user- and
model-facing strings in English.

Details (developed as 0.1.0 snapshot, published as 1.1.0):

### Added
- Embedded catalog: 106 rules (`RULES_YAML` + `PROFILE_SCHEMA_YAML` via
  `include_str!`, no runtime file reads) — REQ-rules-001.
- Serde models (`Severity`, `Rule`, `RulesFile`, `ProfileAnswers`,
  `RulesConfig`) with explicit validation (unknown ID/flag → error) —
  REQ-rules-002, REQ-rules-004.
- Pure resolver (`resolve_active_ruleset` + `group_by_primary_flag` +
  `dedup_by_id`, MUST > SHOULD > AVOID ordering) with 11 tests —
  REQ-rules-004.
- `project init --standard` writes `.dec/rules/transversal.yaml` +
  `profile.schema.yaml` (direct write, bridge-style) and the `project.toml`
  template gains `[rules]` — REQ-rules-001, REQ-rules-005.
- `spec init`: once-only questionnaire (stdin, skipped without TTY) +
  `[PROFILE]` block + resolution + mirror materialization +
  `active-ruleset.md`; `profile update` regenerates everything —
  REQ-rules-002, REQ-rules-005.
- 6 `rules` subcommands (`list`, `search`, `resolve`, `context`, `profile
  update`, `resync`) with stage-aware scoring, 2000-token budget and `--json`
  on all — REQ-rules-003, REQ-rules-006.
- `execute_task` agent hook: researcher writes `{{task_id}}-rules.md`, coder
  includes it (absent → `RULES_CONTEXT_MISSING`, never aborts), reviewer
  pre-triage (`RULES_SUSPECT`) + blocking §7.1 set + `RULES_GATE` markers
  (`exit 1` on FAIL, SKIP without `.dec/rules`), documenter logs exceptions
  to `decisions/` — REQ-rules-007, REQ-rules-008.
- README section (`Development Rules`) and this CHANGELOG.

### Verified
- `cargo build --release` clean, `cargo test` fully green,
  `cargo clippy -- -D warnings` with 0 warnings, installed binary serves
  `dectl rules --help`.
