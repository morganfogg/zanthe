use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::io;

pub struct GameError {
    kind: GameErrorKind,
    detail: Option<String>,
}

pub enum GameErrorKind {
    VersionSix,
    InvalidFile,
    InvalidOperation(String),
    IOError(io::Error),
}

impl GameError {
    pub fn invalid_operation<T: Into<String>>(value: T) -> Self {
        GameError {
            kind: GameErrorKind::InvalidOperation(value.into()),
            detail: None,
        }
    }

    pub fn invalid_file() -> Self {
        GameError {
            kind: GameErrorKind::InvalidFile,
            detail: None,
        }
    }

    pub fn version_six() -> Self {
        GameError {
            kind: GameErrorKind::VersionSix,
            detail: None,
        }
    }

    pub fn io_error(inner: io::Error) -> Self {
        GameError {
            kind: GameErrorKind::IOError(inner),
            detail: None,
        }
    }
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self.kind {
                GameErrorKind::VersionSix => "Version 6 story files are not supported".to_string(),
                GameErrorKind::InvalidFile => {
                    "The file you have specified is not a supported Z-Code file".to_string()
                }
                GameErrorKind::InvalidOperation(e) => {
                    format!("Error while running game: {}", e)
                }
                GameErrorKind::IOError(e) => {
                    format!("I/O Error: {}", e)
                }
            }
        )
    }
}

impl Debug for GameError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Error for GameError {}

impl From<io::Error> for GameError {
    fn from(other: io::Error) -> GameError {
        GameError::io_error(other)
    }
}
