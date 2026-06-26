use anyhow::{Context, Result};
use colored::Colorize;
use is_terminal::IsTerminal;
use serde::Serialize;
use std::fmt::Write;
use std::io::{self, Write as IoWrite};

use crate::bail_app_err;
use crate::core::db::{get_db, MemoryEntry, Storage};
use crate::core::output::{palette, OutputMode};

#[derive(Debug, Serialize)]
pub struct MemoryShowOutput {
    pub entry: MemoryEntry,
}

fn fetch_memory_list() -> Result<Vec<(i64, String)>> {
    let db = get_db()?;
    db.query_map(
        "SELECT id, content FROM memories WHERE deleted_at IS NULL ORDER BY id DESC LIMIT 50",
        &[],
        |row| {
            let id: i64 = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((id, content))
        },
    )
}

fn preview(content: &str, max: usize) -> String {
    let trimmed = content.trim();
    if trimmed.len() <= max {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..max])
    }
}

fn select_id_interactively(entries: &[(i64, String)]) -> Result<i64> {
    if entries.is_empty() {
        bail_app_err!(
            "No memories found",
            "Add some memories first with `dectl memory add`"
        );
    }

    if entries.len() == 1 {
        return Ok(entries[0].0);
    }

    if which::which("fzf").is_ok() {
        let mut input = String::new();
        for (id, content) in entries {
            writeln!(input, "#{}: {}", id, preview(content, 60))?;
        }

        let mut child = std::process::Command::new("fzf")
            .arg("--prompt=Select memory > ")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(input.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if !output.status.success() {
            bail_app_err!("Selection cancelled");
        }

        let line = String::from_utf8_lossy(&output.stdout);
        let line = line.trim();
        if let Some(rest) = line.strip_prefix('#') {
            if let Some(id_str) = rest.split(':').next() {
                if let Ok(id) = id_str.trim().parse::<i64>() {
                    return Ok(id);
                }
            }
        }
        bail_app_err!("Could not parse selected ID from fzf output");
    } else {
        eprintln!(
            "{}",
            "fzf not found — using numbered selection".color(palette::DIM)
        );
        for (i, (id, content)) in entries.iter().enumerate() {
            println!("{:>3}. #{}: {}", i + 1, id, preview(content, 70));
        }
        eprint!("Enter number or #ID: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if let Some(rest) = input.strip_prefix('#') {
            if let Ok(id) = rest.parse::<i64>() {
                if entries.iter().any(|(eid, _)| *eid == id) {
                    return Ok(id);
                }
            }
        } else if let Ok(num) = input.parse::<usize>() {
            if num >= 1 && num <= entries.len() {
                return Ok(entries[num - 1].0);
            }
        }

        bail_app_err!("Invalid selection");
    }
}

fn run_with_id(db: &impl Storage, id: i64, mode: OutputMode) -> Result<()> {
    let cols = crate::core::db::MEMORY_SELECT_COLS;
    let sql = format!(
        "SELECT {} FROM memories WHERE id = ?1 AND deleted_at IS NULL",
        cols
    );
    let entry = db
        .query_row(&sql, rusqlite::params![id], MemoryEntry::from_row)
        .context(format!("Memory entry #{} not found", id))?;

    let output = MemoryShowOutput { entry };

    mode.print(&output)?;

    Ok(())
}

pub fn run(id: i64, mode: OutputMode) -> Result<()> {
    let db = get_db()?;

    if id == 0 {
        if mode.is_json() || !std::io::stdin().is_terminal() {
            bail_app_err!(
                "Interactive selection requires a TTY and Human output mode",
                "Use `dectl memory show <ID>` with a specific memory ID"
            );
        }
        let entries = fetch_memory_list()?;
        let selected = select_id_interactively(&entries)?;
        return run_with_id(db, selected, mode);
    }

    run_with_id(db, id, mode)
}
