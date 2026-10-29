//! Content profiles (REQ-012 / T061).
//!
//! Two contracts are checked here:
//! 1. the generated guidance for the `core` profile lists exactly the included
//!    documents (with consumer and shape) and the excluded ones (with reason);
//! 2. the `core` fixture is consumed by the declared consumer contract without
//!    adapters: the files sit at the paths the consumer reads and keep the
//!    declared shape.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

#[derive(serde::Deserialize)]
struct ProfilesFile {
    profiles: Vec<ProfileDef>,
}

#[derive(serde::Deserialize)]
struct ProfileDef {
    name: String,
    includes: Vec<IncludedDoc>,
    excludes: Vec<ExcludedDoc>,
    #[serde(default)]
    contract: Option<ContractDef>,
}

#[derive(serde::Deserialize)]
struct ContractDef {
    checklist: Vec<ChecklistRow>,
    non_fit: NonFitDef,
    implementation_guide: GuideDef,
}

#[derive(serde::Deserialize)]
struct ChecklistRow {
    item: String,
    destination: String,
    check: String,
}

#[derive(serde::Deserialize)]
struct NonFitDef {
    owner: String,
    schema_ref: String,
    file: String,
    fields: Vec<NonFitField>,
}

#[derive(serde::Deserialize)]
struct NonFitField {
    name: String,
    meaning: String,
}

#[derive(serde::Deserialize)]
struct GuideDef {
    pattern: String,
    mandatory_tests: Vec<String>,
    coverage: String,
}

#[derive(serde::Deserialize)]
struct IncludedDoc {
    path: String,
    consumer: String,
    shape: String,
}

#[derive(serde::Deserialize)]
struct ExcludedDoc {
    path: String,
    reason: String,
}

fn dectl_bin() -> String {
    env!("CARGO_BIN_EXE_dectl").to_string()
}

fn run_dectl(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(dectl_bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("Failed to execute dectl")
}

fn create_dec_base(tmp: &TempDir) {
    fs::create_dir_all(tmp.path().join(".dec/config")).unwrap();
    fs::create_dir_all(tmp.path().join(".dec/isa")).unwrap();
    fs::create_dir_all(tmp.path().join(".dec/state")).unwrap();
    fs::write(
        tmp.path().join(".dec/config/project.toml"),
        "[project]\nname = \"test-project\"\nproject_type = \"other\"\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".dec/isa/project.isa.md"),
        "# Project ISA\n\n## Identity\nName: test-project\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join(".dec/state/last_session.md"),
        "# Last Session\n\n**Date**: 2026-06-29\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("specs")).unwrap();
    fs::write(
        tmp.path().join("specs/spec.md"),
        "# Master Spec\n\n### REQ-001: Existing Feature\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("specs/tasks.md"),
        "- [x] [T001] Existing — S\n",
    )
    .unwrap();
}

/// Load the declared profile from the embedded YAML source.
fn declared_profile(name: &str) -> ProfileDef {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/content_profiles.yaml");
    let yaml = fs::read_to_string(&path).expect("content_profiles.yaml must be readable");
    let file: ProfilesFile = serde_yaml::from_str(&yaml).expect("content_profiles.yaml must parse");
    file.profiles
        .into_iter()
        .find(|profile| profile.name == name)
        .unwrap_or_else(|| panic!("profile '{name}' must be declared"))
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_profile/core")
}

fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("fixture dir must be readable") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
}

fn relative_to_specs(path: &Path) -> String {
    let root = fixture_root();
    let rel = path
        .strip_prefix(&root)
        .unwrap()
        .to_string_lossy()
        .to_string();
    format!("specs/{rel}")
}

#[test]
fn test_core_profile_generates_exactly_the_declared_documents() {
    let profile = declared_profile("core");

    let mut found: Vec<String> = Vec::new();
    let mut files = Vec::new();
    collect_markdown(&fixture_root(), &mut files);
    for file in &files {
        found.push(relative_to_specs(file));
    }
    found.sort();

    let mut declared: Vec<String> = profile
        .includes
        .iter()
        .map(|doc| doc.path.clone())
        .collect();
    declared.sort();

    assert_eq!(
        found, declared,
        "the core fixture must contain exactly the declared documents"
    );

    let spec = fs::read_to_string(fixture_root().join("spec.md")).unwrap();
    let project_name = spec
        .lines()
        .find(|line| line.starts_with("# "))
        .expect("spec.md must start a level-1 heading")
        .trim_start_matches("# ")
        .trim();
    assert_eq!(
        project_name, "Ledger Pay",
        "the first '# ' line of spec.md declares the project name"
    );
}

#[test]
fn test_core_fixture_matches_the_declared_shape_without_adapters() {
    let profile = declared_profile("core");

    let data_model_doc = profile
        .includes
        .iter()
        .find(|doc| doc.path.ends_with("data-model.md"))
        .expect("core must include data-model.md");
    assert!(
        data_model_doc
            .shape
            .contains("Field / Type / Required / Indexes / Description"),
        "the declared contract must name the data-model columns"
    );

    let cli_doc = profile
        .includes
        .iter()
        .find(|doc| doc.path.ends_with("cli.md"))
        .expect("core must include interface-contracts/cli.md");
    assert!(
        cli_doc.shape.contains("### <METHOD> /<path>"),
        "the declared contract must name the endpoint heading form"
    );
    assert!(
        profile
            .includes
            .iter()
            .all(|doc| !doc.consumer.trim().is_empty()),
        "every included document declares a consumer"
    );

    // data-model.md: every entity table uses exactly the declared columns.
    let data_model = fs::read_to_string(fixture_root().join("data-model.md")).unwrap();
    let lines: Vec<&str> = data_model.lines().collect();
    let mut entity_count = 0;
    for (i, line) in lines.iter().enumerate() {
        if !line.starts_with("### ") {
            continue;
        }
        entity_count += 1;
        let header = lines[i + 1].trim();
        assert_eq!(
            header, "| Field | Type | Required | Indexes | Description |",
            "entity tables must use the declared columns"
        );
    }
    assert!(entity_count > 0, "fixture must declare at least one entity");

    // interface-contracts/cli.md: every endpoint is `### <METHOD> /<path>`.
    let cli = fs::read_to_string(fixture_root().join("interface-contracts/cli.md")).unwrap();
    let methods = ["GET", "POST", "PUT", "PATCH", "DELETE"];
    let mut endpoint_count = 0;
    for line in cli.lines() {
        let Some(rest) = line.strip_prefix("### ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or_default();
        assert!(
            methods.contains(&method),
            "endpoint heading must declare an HTTP method, got: {line}"
        );
        assert!(
            path.starts_with('/'),
            "endpoint heading must declare a path, got: {line}"
        );
        endpoint_count += 1;
    }
    assert!(
        endpoint_count > 0,
        "fixture must declare at least one endpoint"
    );
}

#[test]
fn test_spec_init_core_profile_guide_lists_included_and_excluded() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(&["spec", "init", "--profile", "core"], tmp.path());
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_profile_guide(&stdout, "core");
}

#[test]
fn test_spec_add_feature_scope_honours_core_profile() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(
        &[
            "spec",
            "add",
            "transfers",
            "--scope",
            "feature",
            "--profile",
            "core",
            "--auto",
        ],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("FEATURE SPEC GUIDANCE"));
    assert_profile_guide(&stdout, "core");
    assert!(
        stdout.contains("CONTRACT CHECKLIST")
            && stdout.contains("dectl workflow run spec_contract"),
        "the feature guidance must require the contract checklist workflow:\n{stdout}"
    );
    assert!(
        !stdout.contains("`specs/tasks.md` — append atomic tasks"),
        "the feature guidance must not ask for an excluded document:\n{stdout}"
    );
}

#[test]
fn test_spec_add_module_scope_honours_core_profile() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(
        &[
            "spec",
            "add",
            "billing",
            "--scope",
            "module",
            "--profile",
            "core",
            "--auto",
        ],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("MODULE SPEC GUIDANCE"));
    assert_profile_guide(&stdout, "core");
    assert!(
        stdout.contains("CONTRACT CHECKLIST")
            && stdout.contains("dectl workflow run spec_contract"),
        "the module guidance must require the contract checklist workflow:\n{stdout}"
    );
    assert!(
        !stdout.contains("Create exactly these files under `specs/billing/`"),
        "the module guidance must not fall back to the default five-document set:\n{stdout}"
    );
}

#[test]
fn test_spec_add_without_profile_keeps_full_suite() {
    let tmp = TempDir::new().unwrap();
    create_dec_base(&tmp);

    let output = run_dectl(
        &["spec", "add", "auth", "--scope", "module", "--auto"],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!stdout.contains("[CONTENT PROFILE:"));
    assert!(stdout.contains("Create exactly these files under `specs/auth/`"));
}

fn assert_profile_guide(stdout: &str, name: &str) {
    let profile = declared_profile(name);

    assert!(stdout.contains(&format!("[CONTENT PROFILE: {name}]")));
    assert!(stdout.contains("[/CONTENT PROFILE]"));
    assert!(stdout.contains("Consumer contract:"));
    assert!(stdout.contains("Applies to scope: feature and module"));

    for doc in &profile.includes {
        assert!(
            stdout.contains(&doc.path),
            "included document {} must be listed:\n{stdout}",
            doc.path
        );
        assert!(
            stdout.contains(&doc.shape),
            "included document {} must declare its shape:\n{stdout}",
            doc.path
        );
    }

    for doc in &profile.excludes {
        assert!(
            stdout.contains(&doc.path) && stdout.contains(&doc.reason),
            "excluded document {} must be listed with its reason:\n{stdout}",
            doc.path
        );
    }

    let contract = profile
        .contract
        .as_ref()
        .expect("the core profile must declare a contract");
    assert!(stdout.contains("Spec contract checklist"));
    for row in &contract.checklist {
        assert!(
            stdout.contains(&row.item) && stdout.contains(&row.destination),
            "checklist row {} -> {} must be listed:\n{stdout}",
            row.item,
            row.destination
        );
        assert!(
            stdout.contains(&row.check),
            "checklist row {} must state its check:\n{stdout}",
            row.item
        );
    }
    assert!(stdout.contains(&contract.non_fit.owner));
    assert!(stdout.contains(&contract.non_fit.schema_ref));
    assert!(stdout.contains(&contract.non_fit.file));
    for field in &contract.non_fit.fields {
        assert!(
            stdout.contains(&format!("`{}`", field.name)) && stdout.contains(&field.meaning),
            "non-fit field {} must be listed with its meaning:\n{stdout}",
            field.name
        );
    }
    assert!(stdout.contains("origin (source section)"));
    assert!(stdout.contains("destination (test + code location)"));
    assert!(stdout.contains(&contract.implementation_guide.pattern));
    for test in &contract.implementation_guide.mandatory_tests {
        assert!(
            stdout.contains(test),
            "mandatory test '{test}' must be listed:\n{stdout}"
        );
    }
    assert!(stdout.contains(&contract.implementation_guide.coverage));
}
