use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::error::Error;
use std::fmt;

// Authr error kinds
#[derive(Debug)]
pub enum AuthrError {
    NotFound,
    NotAuthorized,
}

impl IntoResponse for AuthrError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AuthrError::NotFound => (StatusCode::NOT_FOUND, "Not Found"),
            AuthrError::NotAuthorized => (StatusCode::FORBIDDEN, "Not Authorized"),
        }
        .into_response()
    }
}

impl fmt::Display for AuthrError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            AuthrError::NotFound => {
                write!(fmt, "Not Found")
            }
            AuthrError::NotAuthorized => {
                write!(fmt, "Not Authorized")
            }
        }
    }
}

impl Error for AuthrError {
    fn description(&self) -> &str {
        match *self {
            AuthrError::NotFound => "Not Found error",
            AuthrError::NotAuthorized => "Not Authorized error",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            AuthrError::NotFound => None,
            AuthrError::NotAuthorized => None,
        }
    }
}

// Result of a Store operation
pub type StoreResult<T> = Result<T, AuthrError>;
