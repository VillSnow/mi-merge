use std::error::Error;

#[derive(Debug)]
pub enum MiMergeError {
    InvalidNote,
}

impl std::fmt::Display for MiMergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MiMergeError::InvalidNote => {
                write!(f, "invalid note")
            }
        }
    }
}

impl Error for MiMergeError {}
