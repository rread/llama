use derive_more::Display;
use std::{error, io};

// pub type Result<T> = std::result::Result<T, LlamaError>;

#[derive(Debug, Display)]
pub enum OpaiError {
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


impl error::Error for OpaiError {}

impl From<io::Error> for OpaiError {
    fn from(err: io::Error) -> Self {
        OpaiError::Io(err)
    }
}

impl From<reqwest::Error> for OpaiError {
    fn from(err: reqwest::Error) -> Self {
        OpaiError::Network(err)
    }
}

impl From<ini::Error> for OpaiError {
    fn from(err: ini::Error) -> Self {
        OpaiError::Config(err)
    }
}

impl From<rustyline::error::ReadlineError> for OpaiError {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        OpaiError::CliError(err)
    }
}
