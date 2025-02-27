use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io;

use crate::crash;
use crate::internal::exit_code::AppExitCode;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum AppError {
    Io(std::io::Error),
    Other(String),
    NonZeroExit,
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(io) => Display::fmt(io, f),
            Self::Other(s) => Display::fmt(s, f),
            Self::NonZeroExit => Display::fmt("Exited with non-zero exit code", f),
        }
    }
}

impl Error for AppError {}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<String> for AppError {
    fn from(string: String) -> Self {
        Self::Other(string)
    }
}

impl From<&str> for AppError {
    fn from(string: &str) -> Self {
        Self::from(string.to_string())
    }
}

pub trait SilentUnwrap<T> {
    fn silent_unwrap(self, error_code: AppExitCode) -> T;
}

impl<T> SilentUnwrap<T> for AppResult<T> {
    fn silent_unwrap(self, exit_code: AppExitCode) -> T {
        match self {
            Ok(val) => val,
            Err(_) => crash!(exit_code, "An error occurred"),
        }
    }
}
