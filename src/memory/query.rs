use anyhow::Result;
use colored::Colorize;
use rusqlite::Connection;
use serde::Serialize;

use super::db::{DbConn, MemoryEntry};
use crate::core::output::OutputMode;

#[derive(Debug, Serialize)]
pub struct MemoryQueryOutput {
    pub query: String,
    pub entries: Vec<MemoryEntry>,
    pub count: usize,
    pub parsed: ParsedQuery,
}

#[derive(Debug, Serialize)]
pub struct ParsedQuery {
    pub filters: Vec<String>,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Value(String),
    Comparator(String),
    And,
    Or,
    Not,
    OrderBy,
    By,
    Asc,
    Desc,
    Limit,
    Ident(String),
}

fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == ':' {
            continue;
        }
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            let mut word = String::new();
            word.push(ch);
            while let Some(&next) = chars.peek() {
                if next.is_alphanumeric() || next == '_' || next == '-' {
                    word.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            match word.to_uppercase().as_str() {
                "AND" => tokens.push(Token::And),
                "OR" => tokens.push(Token::Or),
                "NOT" => tokens.push(Token::Not),
                "ORDER" => tokens.push(Token::OrderBy),
                "BY" => tokens.push(Token::By),
                "ASC" => tokens.push(Token::Asc),
                "DESC" => tokens.push(Token::Desc),
                "LIMIT" => tokens.push(Token::Limit),
                _ => tokens.push(Token::Ident(word)),
            }
        } else if ch == '>' || ch == '<' || ch == '!' || ch == '=' {
            let mut comp = String::new();
            comp.push(ch);
            if let Some(&'=') = chars.peek() {
                comp.push(chars.next().unwrap());
            }
            tokens.push(Token::Comparator(comp));
        } else if ch == '\'' || ch == '"' {
            let quote = ch;
            let mut val = String::new();
            for next in chars.by_ref() {
                if next == quote {
                    break;
                }
                val.push(next);
            }
            tokens.push(Token::Value(val));
        } else {
            return Err(anyhow::anyhow!("Unexpected character '{}' in query", ch));
        }
    }

    Ok(tokens)
}

fn build_where_clause(
    tokens: &[Token],
    params: &mut Vec<String>,
) -> Result<String> {
    let mut sql_parts = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Not => {
                if i + 1 < tokens.len() {
                    let (expr, consumed) = parse_field_expr(&tokens[i + 1..], params)?;
                    sql_parts.push(format!("NOT ({})", expr));
                    i += consumed;
                } else {
                    return Err(anyhow::anyhow!("NOT must be followed by a field expression"));
                }
            }
            Token::And => {
                sql_parts.push("AND".to_string());
            }
            Token::Or => {
                sql_parts.push("OR".to_string());
            }
            Token::Ident(_field_name) => {
                let (expr, consumed) = parse_field_expr(&tokens[i..], params)?;
                sql_parts.push(expr);
                i += consumed - 1;
            }
            Token::OrderBy | Token::Limit | Token::By | Token::Asc | Token::Desc => {
                break;
            }
            Token::Value(_) | Token::Comparator(_) => {
                return Err(anyhow::anyhow!("Unexpected value or comparator without field"));
            }
        }
        i += 1;
    }

    if sql_parts.is_empty() {
        return Ok("1=1".to_string());
    }

    let mut combined = String::new();
    let mut expect_expr = true;
    for part in &sql_parts {
        let is_op = part == "AND" || part == "OR";
        if is_op && expect_expr {
            return Err(anyhow::anyhow!("Unexpected operator at start of expression"));
        }
        if is_op && !expect_expr {
            if !combined.is_empty() {
                combined.push(' ');
            }
            combined.push_str(part);
            expect_expr = true;
        } else if !is_op && expect_expr {
            if !combined.is_empty() {
                combined.push(' ');
            }
            combined.push_str(part);
            expect_expr = false;
        } else if !is_op && !expect_expr {
            return Err(anyhow::anyhow!("Missing operator between expressions"));
        }
    }

    if expect_expr {
        return Err(anyhow::anyhow!("Query ends with operator"));
    }

    Ok(combined)
}

fn parse_field_expr(tokens: &[Token], params: &mut Vec<String>) -> Result<(String, usize)> {
    if tokens.len() < 2 {
        return Err(anyhow::anyhow!("Incomplete field expression"));
    }

    let field_name = match &tokens[0] {
        Token::Ident(name) => name.to_lowercase(),
        _ => return Err(anyhow::anyhow!("Expected field name")),
    };

    let valid_fields = ["type", "tags", "project", "created"];
    if !valid_fields.contains(&field_name.as_str()) {
        return Err(anyhow::anyhow!(
            "Unknown field '{}'. Valid fields: {}",
            field_name,
            valid_fields.join(", ")
        ));
    }

    let db_field = match field_name.as_str() {
        "type" => "m.type",
        "tags" => "m.tags",
        "project" => "m.project",
        "created" => "m.created_at",
        _ => unreachable!(),
    };

    let consumed;

    let (comparator, raw_value) = if tokens.len() >= 2 {
        let next_is_comparator = matches!(&tokens[1], Token::Comparator(_));
        let next_is_value = matches!(&tokens[1], Token::Value(_));
        let next_is_ident = matches!(&tokens[1], Token::Ident(_));

        if next_is_comparator {
            let comp = match &tokens[1] {
                Token::Comparator(c) => c.clone(),
                _ => unreachable!(),
            };
            if tokens.len() >= 3 {
                let val = match &tokens[2] {
                    Token::Value(v) => v.clone(),
                    Token::Ident(v) => v.clone(),
                    _ => return Err(anyhow::anyhow!("Expected value after comparator")),
                };
                consumed = 3;
                (comp, val)
            } else {
                return Err(anyhow::anyhow!("Expected value after comparator"));
            }
        } else if next_is_value {
            let v = match &tokens[1] {
                Token::Value(v) => v.clone(),
                _ => unreachable!(),
            };
            consumed = 2;
            ("=".to_string(), v)
        } else if next_is_ident {
            let v = match &tokens[1] {
                Token::Ident(v) => v.clone(),
                _ => unreachable!(),
            };
            consumed = 2;
            ("=".to_string(), v)
        } else {
            return Err(anyhow::anyhow!("Expected value after field '{}'", field_name));
        }
    } else {
        return Err(anyhow::anyhow!("Expected value after field '{}'", field_name));
    };

    let param_idx = params.len() + 1;

    if field_name == "tags" {
        params.push(format!("%{}%", raw_value));
        let op = if comparator == "!=" || comparator == "<>" {
            "NOT LIKE"
        } else if comparator == "=" {
            "LIKE"
        } else {
            return Err(anyhow::anyhow!("Invalid comparator '{}' for tags field", comparator));
        };
        Ok((format!("{} {} ?{}", db_field, op, param_idx), consumed))
    } else if field_name == "created" {
        params.push(raw_value);
        Ok((format!("{} {} ?{}", db_field, comparator, param_idx), consumed))
    } else {
        params.push(raw_value);
        let op = if comparator == "=" {
            "="
        } else if comparator == "!=" || comparator == "<>" {
            "!="
        } else {
            return Err(anyhow::anyhow!("Invalid comparator '{}' for field '{}'", comparator, field_name));
        };
        Ok((format!("{} {} ?{}", db_field, op, param_idx), consumed))
    }
}

fn parse_order_limit(tokens: &[Token]) -> Result<(Option<String>, Option<String>, Option<usize>)> {
    let mut order_by = None;
    let mut order_dir = None;
    let mut limit = None;

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::OrderBy => {
                let val_idx = if i + 1 < tokens.len() && tokens[i + 1] == Token::By {
                    i + 2
                } else {
                    i + 1
                };
                if val_idx + 1 < tokens.len() {
                    let field = match &tokens[val_idx] {
                        Token::Ident(f) => f.clone(),
                        _ => return Err(anyhow::anyhow!("Expected field name after ORDER BY")),
                    };
                    let valid_order = ["created", "id"];
                    if !valid_order.contains(&field.as_str()) {
                        return Err(anyhow::anyhow!(
                            "Cannot ORDER BY '{}'. Valid: {}",
                            field,
                            valid_order.join(", ")
                        ));
                    }
                    let db_field = match field.as_str() {
                        "created" => "m.created_at",
                        "id" => "m.id",
                        _ => unreachable!(),
                    };
                    order_by = Some(db_field.to_string());
                    let dir_idx = val_idx + 1;
                    if dir_idx < tokens.len() {
                        match &tokens[dir_idx] {
                            Token::Asc => order_dir = Some("ASC".to_string()),
                            Token::Desc => order_dir = Some("DESC".to_string()),
                            _ => order_dir = Some("DESC".to_string()),
                        }
                    }
                    i += 2;
                } else {
                    return Err(anyhow::anyhow!("Incomplete ORDER BY clause"));
                }
            }
            Token::Limit => {
                if i + 1 < tokens.len() {
                    match &tokens[i + 1] {
                        Token::Ident(n) => {
                            let n: usize = n.parse().map_err(|_| anyhow::anyhow!("Expected number after LIMIT, got '{}'", n))?;
                            limit = Some(n);
                        }
                        _ => return Err(anyhow::anyhow!("Expected number after LIMIT")),
                    }
                    i += 1;
                } else {
                    return Err(anyhow::anyhow!("LIMIT requires a number"));
                }
            }
            _ => {}
        }
        i += 1;
    }

    Ok((order_by, order_dir, limit))
}

pub struct ParsedQueryResult {
    pub where_clause: String,
    pub params: Vec<String>,
    pub order_by: Option<String>,
    pub order_dir: Option<String>,
    pub limit: Option<usize>,
}

pub fn parse_query(query: &str) -> Result<ParsedQueryResult> {
    let tokens = tokenize(query)?;

    let order_start = tokens.iter().position(|t| matches!(t, Token::OrderBy));
    let limit_start = tokens.iter().position(|t| matches!(t, Token::Limit));

    let split_at = std::cmp::min(
        order_start.unwrap_or(tokens.len()),
        limit_start.unwrap_or(tokens.len()),
    );

    let filter_tokens = &tokens[..split_at];
    let order_limit_tokens = &tokens[split_at..];

    let mut params = Vec::new();
    let where_clause = build_where_clause(filter_tokens, &mut params)?;
    let (order_by, order_dir, limit) = parse_order_limit(order_limit_tokens)?;

    Ok(ParsedQueryResult {
        where_clause,
        params,
        order_by,
        order_dir,
        limit,
    })
}

pub fn run(query: String, project: Option<String>, limit: Option<usize>, mode: OutputMode) -> Result<()> {
    let db = DbConn::new()?;
    let cols = super::db::MEMORY_SELECT_COLS;

    let parsed = parse_query(&query)?;
    let final_limit = limit.or(parsed.limit).unwrap_or(50);
    let order_by = parsed
        .order_by
        .clone()
        .unwrap_or_else(|| "m.created_at".to_string());
    let order_dir = parsed.order_dir.clone().unwrap_or_else(|| "DESC".to_string());

    let entries = query_entries(db.conn(), cols, &parsed, project.as_deref(), final_limit, &order_by, &order_dir);

    let count = entries.len();
    let output = MemoryQueryOutput {
        query,
        entries,
        count,
        parsed: ParsedQuery {
            filters: parsed.params.clone(),
            order_by: Some(order_by),
            order_direction: Some(order_dir),
            limit: Some(final_limit),
        },
    };

    match mode {
        OutputMode::Json => {
            let envelope = crate::core::output::JsonEnvelope::ok(&output);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputMode::Human => {
            if output.count == 0 {
                println!("No results found.");
                return Ok(());
            }
            println!("Found {} result(s):\n", count);
            for entry in &output.entries {
                println!(
                    "[{}] {} ({})",
                    entry.id,
                    entry.content.chars().take(60).collect::<String>().green(),
                    entry.type_.dimmed()
                );
                if !entry.tags.is_empty() {
                    println!("  Tags: {}", entry.tags.join(", ").cyan());
                }
                if let Some(ref p) = entry.project {
                    println!("  Project: {}", p);
                }
                println!();
            }
        }
    }

    Ok(())
}

fn query_entries(
    conn: &Connection,
    cols: &str,
    parsed: &ParsedQueryResult,
    project: Option<&str>,
    limit: usize,
    order_by: &str,
    order_dir: &str,
) -> Vec<MemoryEntry> {
    let project_filter = if project.is_some() {
        format!("AND m.project = ?{}", parsed.params.len() + 1)
    } else {
        String::new()
    };

    let sql = format!(
        "SELECT {} FROM memories m WHERE m.deleted_at IS NULL AND ({}) {} ORDER BY {} {} LIMIT ?{}",
        cols,
        parsed.where_clause,
        project_filter,
        order_by,
        order_dir,
        if project.is_some() { parsed.params.len() + 2 } else { parsed.params.len() + 1 }
    );

    let mut stmt = conn.prepare(&sql).expect("Failed to prepare query");

    let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    for p in &parsed.params {
        all_params.push(Box::new(p.clone()));
    }
    if let Some(proj) = project {
        all_params.push(Box::new(proj.to_string()));
    }
    all_params.push(Box::new(limit as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = all_params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt
        .query_map(param_refs.as_slice(), MemoryEntry::from_row)
        .expect("Failed to execute query");
    rows.filter_map(|r| r.ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("type:decision").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0], Token::Ident(_)));
        assert!(matches!(&tokens[1], Token::Ident(_)));
    }

    #[test]
    fn test_tokenize_with_and() {
        let tokens = tokenize("type:decision AND tags:rust").unwrap();
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_tokenize_with_order_limit() {
        let tokens = tokenize("type:decision ORDER BY created DESC LIMIT 10").unwrap();
        assert!(tokens.contains(&Token::OrderBy));
        assert!(tokens.contains(&Token::Desc));
        assert!(tokens.contains(&Token::Limit));
        assert!(tokens.contains(&Token::Ident("10".to_string())));
    }

    #[test]
    fn test_parse_simple_type_filter() {
        let result = parse_query("type:decision").unwrap();
        assert!(result.where_clause.contains("m.type"));
        assert_eq!(result.params.len(), 1);
        assert_eq!(result.params[0], "decision");
    }

    #[test]
    fn test_parse_tags_filter() {
        let result = parse_query("tags:architecture").unwrap();
        assert!(result.where_clause.contains("LIKE"));
        assert_eq!(result.params.len(), 1);
        assert_eq!(result.params[0], "%architecture%");
    }

    #[test]
    fn test_parse_boolean_and() {
        let result = parse_query("type:decision AND tags:architecture").unwrap();
        assert!(result.where_clause.contains("AND"));
        assert_eq!(result.params.len(), 2);
    }

    #[test]
    fn test_parse_order_limit() {
        let result = parse_query("type:decision ORDER BY created DESC LIMIT 5").unwrap();
        assert_eq!(result.limit, Some(5));
        assert!(result.order_by.is_some());
    }

    #[test]
    fn test_parse_not_operator() {
        let result = parse_query("NOT type:incident").unwrap();
        assert!(result.where_clause.contains("NOT"));
    }

    #[test]
    fn test_parse_or_operator() {
        let result = parse_query("type:decision OR type:research").unwrap();
        assert!(result.where_clause.contains("OR"));
    }

    #[test]
    fn test_parse_invalid_field() {
        let result = parse_query("invalid:value");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_created_comparator() {
        let result = parse_query("created:>2026-01-01").unwrap();
        assert!(result.where_clause.contains(">"));
        assert_eq!(result.params[0], "2026-01-01");
    }

    #[test]
    fn test_parse_empty_query() {
        let result = parse_query("").unwrap();
        assert_eq!(result.where_clause, "1=1");
    }

    #[test]
    fn test_parse_project_filter() {
        let result = parse_query("project:dectl").unwrap();
        assert_eq!(result.params[0], "dectl");
    }
}
