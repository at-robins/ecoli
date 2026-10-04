//! This module defines specific error types.

use getset::{CopyGetters, Getters};
use log::error;

/// An application wide error.
#[derive(Debug, Clone, Getters, CopyGetters)]
pub struct ApplicationError {
    /// The error type.
    #[getset(get_copy = "pub")]
    error_type: ApplicationErrorType,
    /// The message stack for logging or display.
    #[getset(get = "pub")]
    internal_messages: Vec<String>,
}

/// An application wide error type.
#[derive(Debug, Copy, Clone)]
pub enum ApplicationErrorType {
    /// An generic error implying an internal problem.
    InternalError,
    /// An input or output related error.
    IOError,
    /// Input data error.
    InputDataError,
    /// Input operation error.
    OutputOperationError,
}

impl std::fmt::Display for ApplicationErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ApplicationErrorType::InternalError => "Generic internal error",
            ApplicationErrorType::IOError => "IO error",
            ApplicationErrorType::InputDataError => "Invalid input data",
            ApplicationErrorType::OutputOperationError => "Output operation error",
        };
        write!(f, "{}", name)
    }
}

impl ApplicationError {
    /// Creates a new error and automatically logs the error.
    pub fn new<T: ToString>(error_type: ApplicationErrorType, message: T) -> Self {
        Self {
            error_type,
            internal_messages: vec![message.to_string()],
        }
    }

    /// Returns the formated internal message stack.
    fn format_internal_messages(&self) -> String {
        self.internal_messages().iter().rev().enumerate().fold(
            String::new(),
            |mut acc, (index, message)| {
                let formatted_message = format!("{:03}: {}\n", index, message);
                acc.push_str(&formatted_message);
                acc
            },
        )
    }

    /// Adds another internal error message to the message stack.
    ///
    /// # Parameters
    ///
    /// * `message` - the message to add to the message stack
    pub fn chain<T: ToString>(mut self, message: T) -> Self {
        self.internal_messages.push(message.to_string());
        self
    }

    /// Logs the error on its default level.
    pub fn log_default(&self) {
        ApplicationErrorLogger::new(self).log_default();
    }
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.error_type(), self.format_internal_messages())
    }
}

impl std::error::Error for ApplicationError {}

impl AsRef<ApplicationError> for ApplicationError {
    fn as_ref(&self) -> &ApplicationError {
        self
    }
}

impl From<std::io::Error> for ApplicationError {
    fn from(error: std::io::Error) -> Self {
        Self::new(ApplicationErrorType::IOError, error)
    }
}

impl From<log::SetLoggerError> for ApplicationError {
    fn from(error: log::SetLoggerError) -> Self {
        Self::new(ApplicationErrorType::InternalError, error)
    }
}

impl From<clap::Error> for ApplicationError {
    fn from(error: clap::Error) -> Self {
        Self::new(ApplicationErrorType::InternalError, error)
    }
}

impl From<core::num::ParseIntError> for ApplicationError {
    fn from(error: core::num::ParseIntError) -> Self {
        Self::new(ApplicationErrorType::InternalError, error)
    }
}

/// A logger for a specific [`SeqError`].
pub struct ApplicationErrorLogger {
    message: String,
    error_type: ApplicationErrorType,
}

impl ApplicationErrorLogger {
    /// Creates a new [`SeqErrorLogger`] from an [`SeqError`].
    ///
    /// # Parameters
    ///
    /// * `error` - the error to create a logger for
    pub fn new<T: AsRef<ApplicationError>>(error: T) -> Self {
        ApplicationErrorLogger {
            message: error.as_ref().to_string(),
            error_type: error.as_ref().error_type(),
        }
    }

    /// Logs the error on its default level.
    pub fn log_default(&self) {
        match self.error_type {
            _ => error!("{}", self.message),
        }
    }
}
