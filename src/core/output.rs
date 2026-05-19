use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

impl OutputMode {
    pub fn from_json_flag(json: bool) -> Self {
        if json {
            OutputMode::Json
        } else {
            OutputMode::Human
        }
    }

    pub fn is_json(&self) -> bool {
        matches!(self, OutputMode::Json)
    }
}

#[derive(Debug, Serialize)]
pub struct JsonEnvelope<T> {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl<T> JsonEnvelope<T> {
    pub fn ok(data: T) -> Self {
        JsonEnvelope {
            status: "ok".to_string(),
            data: Some(data),
            message: None,
            hint: None,
        }
    }

    pub fn error(message: impl Into<String>, hint: Option<impl Into<String>>) -> Self {
        JsonEnvelope {
            status: "error".to_string(),
            data: None,
            message: Some(message.into()),
            hint: hint.map(|h| h.into()),
        }
    }
}

pub struct Output;

impl Output {
    pub fn print<T: Serialize>(data: &T, mode: OutputMode) {
        match mode {
            OutputMode::Json => {
                let envelope = JsonEnvelope::ok(data);
                println!("{}", serde_json::to_string(&envelope).unwrap());
            }
            OutputMode::Human => {
                println!("{}", serde_json::to_string_pretty(data).unwrap());
            }
        }
    }

    pub fn print_error(message: &str, hint: Option<&str>, mode: OutputMode) {
        match mode {
            OutputMode::Json => {
                let envelope: JsonEnvelope<()> = JsonEnvelope::error(message, hint);
                eprintln!("{}", serde_json::to_string(&envelope).unwrap());
            }
            OutputMode::Human => {
                let err_msg = format!("Error: {}", message.red());
                if let Some(h) = hint {
                    eprintln!("{} {}", err_msg, format!("({})", h).dimmed());
                } else {
                    eprintln!("{}", err_msg);
                }
            }
        }
    }

    pub fn print_success(message: &str, mode: OutputMode) {
        match mode {
            OutputMode::Json => {
                #[derive(Serialize)]
                struct Msg {
                    message: String,
                }
                let envelope = JsonEnvelope::ok(Msg {
                    message: message.to_string(),
                });
                println!("{}", serde_json::to_string(&envelope).unwrap());
            }
            OutputMode::Human => {
                println!("{}", message.green());
            }
        }
    }
}
