use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

fn parse_doc(path: &Path) -> Result<toml::map::Map<String, toml::Value>> {
    let content = fs::read_to_string(path).context("Failed to read TOML file")?;
    Ok(toml::from_str(&content).unwrap_or_else(|_| toml::map::Map::new()))
}

fn write_doc(path: &Path, doc: &toml::map::Map<String, toml::Value>) -> Result<()> {
    let new_content = toml::to_string_pretty(doc).context("Failed to serialize TOML file")?;
    fs::write(path, &new_content).context("Failed to write TOML file")?;
    Ok(())
}

fn ensure_table_path<'a>(
    doc: &'a mut toml::map::Map<String, toml::Value>,
    key_path: &str,
) -> &'a mut toml::map::Map<String, toml::Value> {
    let parts: Vec<&str> = key_path.splitn(2, '.').collect();
    if parts.len() == 1 {
        doc.entry(parts[0].to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
            .as_table_mut()
            .expect("entry is always a table")
    } else {
        let inner = doc
            .entry(parts[0].to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        let inner_table = inner.as_table_mut().expect("entry is always a table");
        ensure_table_path(inner_table, parts[1])
    }
}

pub fn update_field(path: &Path, key: &str, value: &str) -> Result<()> {
    let mut doc = parse_doc(path)?;
    let parts: Vec<&str> = key.rsplitn(2, '.').collect();
    if parts.len() == 1 {
        doc.insert(key.to_string(), toml::Value::String(value.to_string()));
    } else {
        let table = ensure_table_path(&mut doc, parts[1]);
        table.insert(parts[0].to_string(), toml::Value::String(value.to_string()));
    }
    write_doc(path, &doc)
}

pub fn merge_array(path: &Path, key: &str, items: &[String]) -> Result<()> {
    let mut doc = parse_doc(path)?;

    let parts: Vec<&str> = key.rsplitn(2, '.').collect();
    let (leaf, parent_path) = if parts.len() == 1 {
        (parts[0], "")
    } else {
        (parts[0], parts[1])
    };

    if parent_path.is_empty() {
        let existing: Vec<String> = doc
            .get(leaf)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let mut merged: Vec<String> = existing;
        for item in items {
            if !merged.contains(item) {
                merged.push(item.clone());
            }
        }
        merged.sort();

        doc.insert(
            leaf.to_string(),
            toml::Value::Array(merged.into_iter().map(toml::Value::String).collect()),
        );
    } else {
        let table = ensure_table_path(&mut doc, parent_path);
        let existing: Vec<String> = table
            .get(leaf)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let mut merged: Vec<String> = existing;
        for item in items {
            if !merged.contains(item) {
                merged.push(item.clone());
            }
        }
        merged.sort();

        table.insert(
            leaf.to_string(),
            toml::Value::Array(merged.into_iter().map(toml::Value::String).collect()),
        );
    }

    write_doc(path, &doc)
}

pub fn ensure_section(path: &Path, key: &str) -> Result<()> {
    let mut doc = parse_doc(path)?;
    ensure_table_path(&mut doc, key);
    write_doc(path, &doc)
}
