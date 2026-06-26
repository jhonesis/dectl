use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

use crate::core::output::OutputMode;

#[derive(Debug)]
pub struct ExecResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub first_error: Option<(usize, String)>,
}

#[derive(serde::Serialize)]
struct ExecOutput {
    total: usize,
    succeeded: usize,
    failed: usize,
    error_line: Option<usize>,
    error_message: Option<String>,
}

pub fn run(path: &Path, mode: OutputMode) -> Result<()> {
    let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
    let reader = io::BufReader::new(file);

    let dectl_path = std::env::current_exe()
        .ok()
        .with_context(|| "Failed to determine dectl path")?;

    let mut results = ExecResult {
        total: 0,
        succeeded: 0,
        failed: 0,
        first_error: None,
    };

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.context("Failed to read line")?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        results.total += 1;

        let command_to_run = if trimmed.starts_with("dectl") {
            trimmed[6..].trim().to_string()
        } else {
            trimmed.to_string()
        };

        let mut cmd = Command::new(&dectl_path);
        cmd.args(command_to_run.split_whitespace());

        let output = cmd.output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    results.succeeded += 1;
                } else {
                    results.failed += 1;
                    if results.first_error.is_none() {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        results.first_error =
                            Some((line_num + 1, format!("Command failed: {}", stderr)));
                    }
                    if results
                        .first_error
                        .as_ref()
                        .map(|(n, _)| *n == line_num + 1)
                        .unwrap_or(false)
                    {
                        eprintln!(
                            "Error: command at line {} failed with exit code {:?}",
                            line_num + 1,
                            result.status.code()
                        );
                        break;
                    }
                }
            }
            Err(e) => {
                results.failed += 1;
                if results.first_error.is_none() {
                    results.first_error = Some((line_num + 1, format!("Failed to execute: {}", e)));
                }
                eprintln!(
                    "Error: failed to execute command at line {}: {}",
                    line_num + 1,
                    e
                );
                break;
            }
        }
    }

    let exec_output = ExecOutput {
        total: results.total,
        succeeded: results.succeeded,
        failed: results.failed,
        error_line: results.first_error.as_ref().map(|(n, _)| *n),
        error_message: results.first_error.as_ref().map(|(_, m)| m.clone()),
    };
    mode.print(&exec_output)?;

    if results.failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
