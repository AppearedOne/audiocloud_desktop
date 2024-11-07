use std::fmt;

#[derive(Debug, Clone)]
pub struct Error {
    e: ErrorType,
}
impl Error {
    pub fn new(t: ErrorType) -> Self {
        Error { e: t }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Parse,
    Connection,
    JSON,
    FileOpen,
    FileSave,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Errored")
    }
}
