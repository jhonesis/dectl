use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

pub fn interpolate(template: &str, vars: &HashMap<String, String>) -> Result<String> {
    let var_pattern = Regex::new(r"\\?\{\{([^}]+)\}\}")
        .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;

    let mut result = String::with_capacity(template.len());
    let mut last_end = 0;

    for cap in var_pattern.captures_iter(template) {
        let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
        result.push_str(&template[last_end..cap.get(0).map(|m| m.start()).unwrap_or(0)]);

        if let Some(stripped) = full_match.strip_prefix('\\') {
            result.push_str(stripped);
        } else {
            if let Some(var_name) = cap.get(1).map(|m| m.as_str().trim()) {
                if let Some(value) = vars.get(var_name) {
                    result.push_str(value);
                } else {
                    anyhow::bail!(
                        "Variable '{{{{}}}}' not found in inputs. Available: {:?}",
                        var_name
                    );
                }
            }
        }
        last_end = cap.get(0).map(|m| m.end()).unwrap_or(last_end);
    }

    result.push_str(&template[last_end..]);
    Ok(result)
}

pub fn extract_variables(template: &str) -> Vec<String> {
    let var_pattern = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
    let mut vars: Vec<String> = Vec::new();

    for cap in var_pattern.captures_iter(template) {
        if let Some(var_name) = cap.get(1) {
            let var_name = var_name.as_str().trim().to_string();
            if !vars.contains(&var_name) {
                vars.push(var_name);
            }
        }
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_interpolation() {
        let vars: HashMap<_, _> = [("name".to_string(), "Alice".to_string())]
            .into_iter()
            .collect();

        let result = interpolate("Hello {{name}}!", &vars).unwrap();
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_multiple_variables() {
        let vars: HashMap<_, _> = [
            ("name".to_string(), "Bob".to_string()),
            ("action".to_string(), "coding".to_string()),
        ]
        .into_iter()
        .collect();

        let result = interpolate("{{name}} loves {{action}}", &vars).unwrap();
        assert_eq!(result, "Bob loves coding");
    }

    #[test]
    fn test_escape_sequence() {
        let vars: HashMap<_, _> = [("name".to_string(), "test".to_string())]
            .into_iter()
            .collect();

        let result = interpolate("Raw {{name}} and escaped \\{{name}}", &vars).unwrap();
        assert_eq!(result, "Raw test and escaped {{name}}");
    }

    #[test]
    fn test_missing_variable() {
        let vars: HashMap<String, String> = HashMap::new();

        let result = interpolate("Hello {{name}}!", &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_variables() {
        let template = "Feature: {{feature_name}} in module {{module}}";
        let vars = extract_variables(template);

        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"feature_name".to_string()));
        assert!(vars.contains(&"module".to_string()));
    }

    #[test]
    fn test_no_duplicates() {
        let template = "{{name}} and {{name}} again";
        let vars = extract_variables(template);

        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0], "name");
    }

    #[test]
    fn test_whitespace_handling() {
        let vars: HashMap<_, _> = [("name".to_string(), "Cris".to_string())]
            .into_iter()
            .collect();

        let result = interpolate("Hello {{  name  }}!", &vars).unwrap();
        assert_eq!(result, "Hello Cris!");
    }
}
