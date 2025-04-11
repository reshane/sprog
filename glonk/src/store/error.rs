use std::error::Error;
use std::fmt;

// Store error kinds
#[derive(Debug)]
pub enum StoreError {
    NotCreated,
    NotFound,
}

impl fmt::Display for StoreError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            StoreError::NotCreated => {
                write!(fmt, "The data could not be created",)
            }
            StoreError::NotFound => {
                write!(fmt, "The data could not be found",)
            }
        }
    }
}

impl Error for StoreError {
    fn description(&self) -> &str {
        match *self {
            StoreError::NotCreated => "NotCreated error",
            StoreError::NotFound => "NotFound error",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            StoreError::NotCreated => None,
            StoreError::NotFound => None,
        }
    }
}

// Result of a Store operation
pub type StoreResult<T> = Result<T, StoreError>;
