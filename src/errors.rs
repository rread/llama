use derive_more::Display;
use std::{error, io};

// pub type Result<T> = std::result::Result<T, LlamaError>;

#[derive(Debug, Display)]
pub enum LlamaError {
    #[display("Http Error: {}", _0)]
    Http(reqwest::StatusCode),
    #[display("IO Error: {}", _0)]
    Io(io::Error),
    #[display("Network Error: {}", _0)]
    Network(reqwest::Error),
    #[display("Config Error: {}", _0)]
    Config(ini::Error),
    #[display("CLI Error: {}", _0)]
    CliError(rustyline::error::ReadlineError),
}


impl error::Error for LlamaError {}

impl From<io::Error> for LlamaError {
    fn from(err: io::Error) -> Self {
        LlamaError::Io(err)
    }
}

impl From<reqwest::Error> for LlamaError {
    fn from(err: reqwest::Error) -> Self {
        LlamaError::Network(err)
    }
}

impl From<ini::Error> for LlamaError {
    fn from(err: ini::Error) -> Self {
        LlamaError::Config(err)
    }
}

impl From<rustyline::error::ReadlineError> for LlamaError {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        LlamaError::CliError(err)
    }
}
