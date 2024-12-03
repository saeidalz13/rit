use core::num;

use crate::utils::colorutils::{BOLD_RED, FORMAT_RESET};

pub enum CliError {
    IOError(std::io::Error),
    ParseIntError(num::ParseIntError),
    GeneralError(&'static str),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::IOError(err) => write!(f, "{}IO Error:{} {}", BOLD_RED, FORMAT_RESET, err),
            CliError::ParseIntError(err) => {
                write!(f, "{}ParseInt Error:{} {}", BOLD_RED, FORMAT_RESET, err)
            }
            CliError::GeneralError(msg) => write!(f, "{}Error:{} {}", BOLD_RED, FORMAT_RESET, msg),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::IOError(err)
    }
}

impl From<num::ParseIntError> for CliError {
    fn from(err: num::ParseIntError) -> Self {
        CliError::ParseIntError(err)
    }
}
