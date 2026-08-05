use anyhow::{Context, Result};
use handlebars::{
    Context as HbContext, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext,
    Renderable,
};
use regex::Regex;
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct IfEqHelper;

impl HelperDef for IfEqHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc HbContext,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let a = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");
        let b = h.param(1).and_then(|p| p.value().as_str()).unwrap_or("");
        let tmpl = if a == b { h.template() } else { h.inverse() };
        match tmpl {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}

pub fn interpolate(template: &str, vars: &HashMap<String, String>) -> Result<String> {
    let referenced = extract_variables(template);
    for var in &referenced {
        if !vars.contains_key(var.as_str()) {
            anyhow::bail!(
                "Variable '{{{{{}}}}}' not found in inputs. Available: {:?}",
                var,
                vars.keys().collect::<Vec<_>>()
            );
        }
    }

    let mut handlebars = handlebars::Handlebars::new();
    handlebars.register_escape_fn(|s: &str| s.to_string());
    handlebars.register_helper("if_eq", Box::new(IfEqHelper));

    let context = serde_json::to_value(vars)
        .map_err(|e| anyhow::anyhow!("Failed to serialize variables: {}", e))?;

    handlebars
        .render_template(template, &context)
        .with_context(|| "Template interpolation error".to_string())
}

pub fn extract_variables(template: &str) -> Vec<String> {
    let var_pattern = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
    let mut vars: Vec<String> = Vec::new();
    let keywords = [
        "if", "if_eq", "else", "each", "with", "unless", "log", "lookup", "this", ".",
    ];

    for cap in var_pattern.captures_iter(template) {
        if let Some(inner) = cap.get(1) {
            let mut content = inner.as_str().trim();
            content = content.strip_prefix('#').unwrap_or(content);
            content = content.strip_prefix('/').unwrap_or(content);
            content = content.strip_prefix('>').unwrap_or(content);
            content = content.strip_prefix('!').unwrap_or(content);
            let content = content.trim();

            if content.is_empty() || keywords.contains(&content) {
                continue;
            }

            let parts: Vec<&str> = content.splitn(2, char::is_whitespace).collect();
            let var_name = if parts.len() >= 2 && keywords.contains(&parts[0]) {
                parts[1].split_whitespace().next().unwrap_or("")
            } else {
                parts[0]
            };

            if !var_name.is_empty() && !vars.contains(&var_name.to_string()) {
                vars.push(var_name.to_string());
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

    #[test]
    fn test_handlebars_if_helper() {
        let vars: HashMap<_, _> = [
            ("body".to_string(), "test payload".to_string()),
            ("method".to_string(), "POST".to_string()),
            ("endpoint".to_string(), "api/test".to_string()),
        ]
        .into_iter()
        .collect();

        let template = "{{#if body}}curl -X {{method}} {{endpoint}}{{/if}}";
        let result = interpolate(template, &vars).unwrap();
        assert_eq!(result, "curl -X POST api/test");
    }

    #[test]
    fn test_handlebars_if_helper_false() {
        let vars: HashMap<_, _> = [
            ("body".to_string(), "".to_string()),
            ("method".to_string(), "GET".to_string()),
            ("endpoint".to_string(), "api/health".to_string()),
        ]
        .into_iter()
        .collect();

        let template = "{{#if body}}curl -X {{method}} {{endpoint}}{{/if}}";
        let result = interpolate(template, &vars).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_handlebars_variables() {
        let template = "{{#if body}}\n  curl -X {{method}} {{endpoint}}\n{{/if}}";
        let vars = extract_variables(template);

        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&"body".to_string()));
        assert!(vars.contains(&"method".to_string()));
        assert!(vars.contains(&"endpoint".to_string()));
    }

    #[test]
    fn test_extract_skips_keywords() {
        let template = "{{#if body}}{{else}}{{/if}}";
        let vars = extract_variables(template);

        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0], "body");
    }

    #[test]
    fn test_extract_each_helper() {
        let template = "{{#each items}}{{this}}{{/each}}";
        let vars = extract_variables(template);

        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0], "items");
    }

    #[test]
    fn test_handlebars_if_eq_helper() {
        let vars: HashMap<_, _> = [
            ("scope".to_string(), "module".to_string()),
            ("name".to_string(), "auth".to_string()),
        ]
        .into_iter()
        .collect();

        let template =
            "{{#if_eq scope \"module\"}}dir={{name}}{{else}}root={{name}}{{/if_eq}}";
        let result = interpolate(template, &vars).unwrap();
        assert_eq!(result, "dir=auth");
    }

    #[test]
    fn test_handlebars_if_eq_helper_else() {
        let vars: HashMap<_, _> = [
            ("scope".to_string(), "feature".to_string()),
            ("name".to_string(), "login".to_string()),
        ]
        .into_iter()
        .collect();

        let template =
            "{{#if_eq scope \"module\"}}dir={{name}}{{else}}root={{name}}{{/if_eq}}";
        let result = interpolate(template, &vars).unwrap();
        assert_eq!(result, "root=login");
    }

    #[test]
    fn test_extract_if_eq_variable() {
        let template = "{{#if_eq scope \"module\"}}{{name}}{{/if_eq}}";
        let vars = extract_variables(template);

        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"scope".to_string()));
        assert!(vars.contains(&"name".to_string()));
    }
}
