use std::fmt;

#[derive(Debug)]
pub struct Error {
    pub code: &'static str,
    pub message: String,
    pub exit_code: i32,
}

impl Error {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_input",
            message: message.into(),
            exit_code: 2,
        }
    }

    pub fn corrupt(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_record",
            message: message.into(),
            exit_code: 1,
        }
    }

    pub fn io(context: impl AsRef<str>, error: std::io::Error) -> Self {
        Self {
            code: "io_error",
            message: format!("{}: {error}", context.as_ref()),
            exit_code: 1,
        }
    }

    pub fn cancelled() -> Self {
        Self {
            code: "cancelled",
            message: "operation cancelled".into(),
            exit_code: 130,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
