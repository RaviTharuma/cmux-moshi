//! Structured errors for the CLI.

use std::fmt;
use std::io;

/// Recoverable CLI failure with a stable exit code.
#[derive(Debug)]
pub enum Error {
    /// A required host command failed or is missing.
    Command { program: String, message: String },
    /// JSON or text parsing failed.
    Parse(String),
    /// Filesystem error.
    Io(io::Error),
    /// User-facing usage problem (wrong flags or environment).
    Usage(String),
}

impl Error {
    /// Builds a command error.
    pub fn command(program: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Command {
            program: program.into(),
            message: message.into(),
        }
    }

    /// Builds a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    /// Builds a usage error.
    pub fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    /// Process exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            _ => 1,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command { program, message } => write!(f, "{program}: {message}"),
            Self::Parse(message) => write!(f, "{message}"),
            Self::Io(err) => write!(f, "{err}"),
            Self::Usage(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
