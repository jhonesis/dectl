use std::fmt::Write;
use std::io::{self, Write as IoWrite};

use anyhow::Result;
use colored::Colorize;
use is_terminal::IsTerminal;
use serde::Serialize;

use crate::bail_app_err;
use crate::core::db::{get_db, Storage};
use crate::core::output::{palette, OutputMode};

#[derive(Debug, Serialize)]
pub struct MemoryDeleteOutput {
    pub id: i64,
    pub deleted_at: String,
    pub hard: bool,
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

pub fn run(id: i64, hard: bool, non_interactive: bool, mode: OutputMode) -> Result<()> {
    let db = get_db()?;

    let selected_id = if id == 0 {
        if mode.is_json() || !std::io::stdin().is_terminal() {
            bail_app_err!(
                "Interactive selection requires a TTY and Human output mode",
                "Use `dectl memory delete <ID>` with a specific memory ID"
            );
        }
        let entries = fetch_memory_list()?;
        select_id_interactively(&entries)?
    } else {
        id
    };

    let exists: bool = db
        .query_row(
            "SELECT id FROM memories WHERE id = ?1",
            rusqlite::params![selected_id],
            |row| row.get::<_, i64>(0),
        )
        .is_ok();

    if !exists {
        bail_app_err!(
            format!("Memory entry #{} not found", selected_id),
            "Run `dectl memory list` to find valid IDs"
        );
    }

    let now = chrono::Utc::now().to_rfc3339();

    if hard {
        if !non_interactive {
            println!("⚠️  This will permanently delete memory #{}", selected_id);
            println!("{}", "Type 'yes' to confirm:".color(palette::WARNING));

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "yes" {
                println!("{}", "Cancelled.".color(palette::DIM));
                return Ok(());
            }
        }

        db.execute(
            "DELETE FROM memories WHERE id = ?1",
            rusqlite::params![selected_id],
        )?;

        let output = MemoryDeleteOutput {
            id: selected_id,
            deleted_at: now,
            hard: true,
        };

        mode.print(&output)?;
    } else {
        db.execute(
            "UPDATE memories SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, selected_id],
        )?;

        let output = MemoryDeleteOutput {
            id: selected_id,
            deleted_at: now,
            hard: false,
        };

        mode.print(&output)?;
    }

    Ok(())
}
