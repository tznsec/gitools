//! Błędy domenowe silnika generowania struktur.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    UnknownTemplate(String),
    UnknownOption {
        template: String,
        option: String,
    },
    InvalidOptionValue {
        option: String,
        value: String,
        expected: String,
    },
    InvalidProjectName(String),
    TargetNotEmpty(PathBuf),
    TargetInvalid { path: PathBuf, reason: String },
    Io { path: PathBuf, source: io::Error },
    Other(String),
}

impl Error {
    pub fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Error::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownTemplate(id) => write!(
                f,
                "nieznany szablon `{id}` (sprawdź listę: gitools list)"
            ),
            Error::UnknownOption { template, option } => write!(
                f,
                "szablon `{template}` nie ma opcji `{option}`"
            ),
            Error::InvalidOptionValue {
                option,
                value,
                expected,
            } => write!(
                f,
                "nieprawidłowa wartość `{value}` dla opcji `{option}` (oczekiwano: {expected})"
            ),
            Error::InvalidProjectName(name) => write!(
                f,
                "nieprawidłowa nazwa projektu: `{name}` (1-64 znaki, bez \\ / : * ? \" < > | i bez znaków sterujących)"
            ),
            Error::TargetNotEmpty(path) => write!(
                f,
                "katalog `{}` nie jest pusty - użyj --force albo innego katalogu",
                path.display()
            ),
            Error::TargetInvalid { path, reason } => {
                write!(f, "katalog `{}` jest nieprawidłowy: {reason}", path.display())
            }
            Error::Io { path, source } => {
                write!(f, "błąd I/O przy `{}`: {source}", path.display())
            }
            Error::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}