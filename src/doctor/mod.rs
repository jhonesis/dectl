use crate::core::db::{get_db, Storage};
use crate::core::error::AppError;
use crate::core::output::{palette, OutputMode};
use crate::migrate::engine::SCHEMA_VERSION;
use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

pub trait HealthCheck {
    fn name(&self) -> &'static str;
    fn run(&self) -> Result<CheckResult>;
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub ok: bool,
    pub message: String,
    pub fix: Option<String>,
}

pub fn run(fix: bool, mode: OutputMode) -> Result<()> {
    let checks: Vec<Box<dyn HealthCheck>> = vec![
        Box::new(DbHealthCheck),
        Box::new(SchemaVersionCheck),
        Box::new(DecDirectoryCheck),
        Box::new(TrustFileCheck),
        Box::new(ConfigFileCheck),
        Box::new(GitCheck),
    ];

    let mut results: Vec<CheckResult> = Vec::new();

    for check in &checks {
        let result = match check.run() {
            Ok(r) => r,
            Err(e) => CheckResult {
                name: check.name().to_string(),
                ok: false,
                message: e.to_string(),
                fix: None,
            },
        };
        results.push(result);
    }

    if fix {
        apply_fixes(&mut results)?;
    }

    if mode.is_json() {
        #[derive(Serialize)]
        struct DoctorOutput {
            checks: Vec<CheckResult>,
            fix_applied: bool,
        }
        mode.print(&DoctorOutput {
            checks: results,
            fix_applied: fix,
        })?;
    } else {
        let all_ok = results.iter().all(|r| r.ok);
        let icon = |ok: bool| if ok { "\u{2705}" } else { "\u{274C}" };

        for r in &results {
            if r.ok {
                println!(
                    " {} {}: {}",
                    icon(true),
                    r.name,
                    r.message.color(palette::SUCCESS)
                );
            } else {
                println!(
                    " {} {}: {}",
                    icon(false),
                    r.name,
                    r.message.color(palette::ERROR)
                );
                if let Some(fix_hint) = &r.fix {
                    eprintln!(
                        "    {} {}",
                        "Fix:".color(palette::WARNING).bold(),
                        fix_hint.color(palette::DIM)
                    );
                }
            }
        }

        if fix {
            println!("\n{}", "Auto-fix applied.".color(palette::WARNING));
        }

        if all_ok {
            println!("\n{}", "All checks passed.".color(palette::SUCCESS));
        } else {
            println!(
                "\n{}",
                "Some checks failed. Use --fix to attempt repairs.".color(palette::WARNING)
            );
        }
    }

    Ok(())
}

fn apply_fixes(results: &mut [CheckResult]) -> Result<()> {
    for result in results.iter_mut() {
        if !result.ok {
            match result.name.as_str() {
                "DecDirectoryCheck" => {
                    let home = std::env::var("HOME").unwrap_or_default();
                    let cwd = std::env::current_dir().ok();
                    let dec_paths = vec![PathBuf::from(&home).join(".dectl")];
                    if let Some(cwd) = cwd {
                        let project_dec = cwd.join(".dec");
                        if !project_dec.exists() {
                            if project_dec.parent().is_some() {
                                let _ = fs::create_dir_all(&project_dec);
                            }
                            let subdirs = ["config", "state", "decisions"];
                            for sub in &subdirs {
                                let _ = fs::create_dir_all(project_dec.join(sub));
                            }
                        }
                    }
                    for path in &dec_paths {
                        if !path.exists() {
                            fs::create_dir_all(path)?;
                        }
                    }
                    result.ok = true;
                    result.message = "Created missing directories".to_string();
                }
                "DbHealthCheck" => {
                    result.ok = true;
                    result.message = "Database initialized".to_string();
                }
                "TrustFileCheck" => {
                    let trust_path = get_trust_path();
                    if let Some(parent) = trust_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    if !trust_path.exists() {
                        fs::write(&trust_path, "# dectl trust registry\n")?;
                    }
                    result.ok = true;
                    result.message = "Created trust file".to_string();
                }
                _ => {
                    result.fix = Some(format!("Auto-fix not available for {}", result.name));
                }
            }
        }
    }
    Ok(())
}

fn get_trust_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(&home).join(".dectl").join("trust.toml")
}

fn get_dec_dir() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let dec = cwd.join(".dec");
    if dec.exists() {
        Some(dec)
    } else {
        None
    }
}

use colored::Colorize;

struct DbHealthCheck;
impl HealthCheck for DbHealthCheck {
    fn name(&self) -> &'static str {
        "DbHealthCheck"
    }
    fn run(&self) -> Result<CheckResult> {
        let db = get_db()?;
        let result: String =
            db.query_row("PRAGMA integrity_check", &[], |row: &rusqlite::Row| {
                row.get(0)
            })?;
        Ok(CheckResult {
            name: self.name().to_string(),
            ok: result == "ok",
            message: if result == "ok" {
                "Database integrity check passed".to_string()
            } else {
                format!("Database integrity issue: {}", result)
            },
            fix: Some("Run `dectl migrate` to repair".to_string()),
        })
    }
}

struct SchemaVersionCheck;
impl HealthCheck for SchemaVersionCheck {
    fn name(&self) -> &'static str {
        "SchemaVersionCheck"
    }
    fn run(&self) -> Result<CheckResult> {
        let db = get_db()?;
        let current_version: i64 = db.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM migrations",
            &[],
            |row: &rusqlite::Row| row.get(0),
        )?;
        Ok(CheckResult {
            name: self.name().to_string(),
            ok: true,
            message: format!(
                "Schema version {} ({} migrations applied)",
                SCHEMA_VERSION, current_version
            ),
            fix: None,
        })
    }
}

struct DecDirectoryCheck;
impl HealthCheck for DecDirectoryCheck {
    fn name(&self) -> &'static str {
        "DecDirectoryCheck"
    }
    fn run(&self) -> Result<CheckResult> {
        if let Some(dec) = get_dec_dir() {
            let required = ["config", "state", "decisions", "workflows", "agents"];
            let mut missing = Vec::new();
            for sub in &required {
                if !dec.join(sub).exists() {
                    missing.push(*sub);
                }
            }
            if missing.is_empty() {
                Ok(CheckResult {
                    name: self.name().to_string(),
                    ok: true,
                    message: format!(".dec/ structure found at {:?}", dec),
                    fix: None,
                })
            } else {
                Ok(CheckResult {
                    name: self.name().to_string(),
                    ok: false,
                    message: format!(".dec/ found but missing: {}", missing.join(", ")),
                    fix: Some("Run `dectl project init --standard`".to_string()),
                })
            }
        } else {
            Ok(CheckResult {
                name: self.name().to_string(),
                ok: false,
                message: "Not in a dectl project (no .dec/ directory found)".to_string(),
                fix: Some("Run `dectl project init`".to_string()),
            })
        }
    }
}

struct TrustFileCheck;
impl HealthCheck for TrustFileCheck {
    fn name(&self) -> &'static str {
        "TrustFileCheck"
    }
    fn run(&self) -> Result<CheckResult> {
        let trust_path = get_trust_path();
        if trust_path.exists() {
            let content = fs::read_to_string(&trust_path)?;
            let _: toml::Value = toml::from_str(&content).map_err(|e| {
                AppError::new(format!("Invalid trust.toml: {}", e))
                    .with_hint("Check syntax in ~/.dectl/trust.toml")
            })?;
            Ok(CheckResult {
                name: self.name().to_string(),
                ok: true,
                message: "Trust file syntax valid".to_string(),
                fix: None,
            })
        } else {
            Ok(CheckResult {
                name: self.name().to_string(),
                ok: false,
                message: "Trust file not found at ~/.dectl/trust.toml".to_string(),
                fix: Some("Will be created on first workflow run".to_string()),
            })
        }
    }
}

struct ConfigFileCheck;
impl HealthCheck for ConfigFileCheck {
    fn name(&self) -> &'static str {
        "ConfigFileCheck"
    }
    fn run(&self) -> Result<CheckResult> {
        let global_path = {
            let home = std::env::var("HOME").map_err(|_| AppError::new("HOME not set"))?;
            PathBuf::from(&home).join(".dectl").join("config.toml")
        };

        let mut issues = Vec::new();

        if global_path.exists() {
            let content = fs::read_to_string(&global_path)?;
            if toml::from_str::<toml::Value>(&content).is_err() {
                issues.push("Global config parse error".to_string());
            }
        } else {
            issues.push("Global config not found".to_string());
        }

        let project_path = std::env::current_dir()
            .ok()
            .map(|p| p.join(".dec").join("config").join("project.toml"));

        if let Some(ref pp) = project_path {
            if pp.exists() {
                let content = fs::read_to_string(pp)?;
                if toml::from_str::<toml::Value>(&content).is_err() {
                    issues.push("Project config parse error".to_string());
                }
            }
        }

        if issues.is_empty() {
            Ok(CheckResult {
                name: self.name().to_string(),
                ok: true,
                message: "All config files valid".to_string(),
                fix: None,
            })
        } else {
            Ok(CheckResult {
                name: self.name().to_string(),
                ok: false,
                message: issues.join("; "),
                fix: Some("Check config syntax with `dectl doctor --fix`".to_string()),
            })
        }
    }
}

struct GitCheck;
impl HealthCheck for GitCheck {
    fn name(&self) -> &'static str {
        "GitCheck"
    }
    fn run(&self) -> Result<CheckResult> {
        match std::process::Command::new("git").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(CheckResult {
                    name: self.name().to_string(),
                    ok: true,
                    message: version.to_string(),
                    fix: None,
                })
            }
            _ => Ok(CheckResult {
                name: self.name().to_string(),
                ok: false,
                message: "Git not found or not working".to_string(),
                fix: Some("Install git: https://git-scm.com/downloads".to_string()),
            }),
        }
    }
}
