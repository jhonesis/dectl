use colored::Colorize;
use serde::Serialize;

pub mod palette {
    use colored::Color;

    pub const SUCCESS: Color = Color::Green;
    pub const ERROR: Color = Color::Red;
    pub const WARNING: Color = Color::Yellow;
    pub const DIM: Color = Color::BrightBlack;
}

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

    pub fn print<T: Serialize>(&self, data: &T) -> anyhow::Result<()> {
        match self {
            OutputMode::Human => HumanFormat.print(data),
            OutputMode::Json => JsonFormat.print(data),
        }
    }

    pub fn print_error(&self, message: &str, hint: Option<&str>) -> anyhow::Result<()> {
        match self {
            OutputMode::Human => HumanFormat.print_error(message, hint),
            OutputMode::Json => JsonFormat.print_error(message, hint),
        }
    }

    pub fn print_success(&self, message: &str) -> anyhow::Result<()> {
        match self {
            OutputMode::Human => HumanFormat.print_success(message),
            OutputMode::Json => JsonFormat.print_success(message),
        }
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

pub trait OutputFormat {
    fn print<T: Serialize>(&self, data: &T) -> anyhow::Result<()>;
    fn print_error(&self, message: &str, hint: Option<&str>) -> anyhow::Result<()>;
    fn print_success(&self, message: &str) -> anyhow::Result<()>;
}

pub struct HumanFormat;
pub struct JsonFormat;

impl OutputFormat for HumanFormat {
    fn print<T: Serialize>(&self, data: &T) -> anyhow::Result<()> {
        println!("{}", serde_json::to_string_pretty(data)?);
        Ok(())
    }

    fn print_error(&self, message: &str, hint: Option<&str>) -> anyhow::Result<()> {
        let err_msg = format!("Error: {}", message.color(palette::ERROR));
        if let Some(h) = hint {
            eprintln!("{} {}", err_msg, format!("({})", h).color(palette::DIM));
        } else {
            eprintln!("{}", err_msg);
        }
        Ok(())
    }

    fn print_success(&self, message: &str) -> anyhow::Result<()> {
        println!("{}", message.color(palette::SUCCESS));
        Ok(())
    }
}

impl OutputFormat for JsonFormat {
    fn print<T: Serialize>(&self, data: &T) -> anyhow::Result<()> {
        let envelope = JsonEnvelope::ok(data);
        println!("{}", serde_json::to_string_pretty(&envelope)?);
        Ok(())
    }

    fn print_error(&self, message: &str, hint: Option<&str>) -> anyhow::Result<()> {
        let envelope: JsonEnvelope<()> = JsonEnvelope::error(message, hint);
        eprintln!("{}", serde_json::to_string_pretty(&envelope)?);
        Ok(())
    }

    fn print_success(&self, message: &str) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Msg {
            message: String,
        }
        let envelope = JsonEnvelope::ok(Msg {
            message: message.to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&envelope)?);
        Ok(())
    }
}

pub struct Output;

impl Output {
    pub fn print<T: Serialize>(data: &T, mode: OutputMode) {
        let _ = mode.print(data);
    }

    pub fn print_error(message: &str, hint: Option<&str>, mode: OutputMode) {
        let _ = mode.print_error(message, hint);
    }

    pub fn print_success(message: &str, mode: OutputMode) {
        let _ = mode.print_success(message);
    }
}
