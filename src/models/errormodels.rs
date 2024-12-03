use core::num;

pub enum CliError {
    IOError(std::io::Error),
    ParseIntError(num::ParseIntError),
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
