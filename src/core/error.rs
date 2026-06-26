use std::fmt;

#[derive(Debug)]
pub struct AppError {
    pub message: String,
    pub hint: Option<String>,
    pub exit_code: i32,
}

impl AppError {
    pub fn new(message: impl Into<String>) -> Self {
        AppError {
            message: message.into(),
            hint: None,
            exit_code: 1,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    #[allow(dead_code)]
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

pub fn extract_error_details(err: &anyhow::Error) -> (Option<&str>, i32) {
    if let Some(app_err) = err.downcast_ref::<AppError>() {
        (app_err.hint.as_deref(), app_err.exit_code)
    } else {
        (None, 1)
    }
}

pub fn exit_for_error(err: anyhow::Error, mode: crate::core::output::OutputMode) -> ! {
    let (hint, exit_code) = extract_error_details(&err);
    crate::core::output::Output::print_error(&err.to_string(), hint, mode);
    std::process::exit(exit_code);
}

#[macro_export]
macro_rules! bail_app_err {
    ($msg:expr) => {
        anyhow::bail!($crate::core::error::AppError::new($msg))
    };
    ($msg:expr, $hint:expr) => {
        anyhow::bail!($crate::core::error::AppError::new($msg).with_hint($hint))
    };
}
