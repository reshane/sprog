use serde::Deserialize;
use std::fmt;

mod user;
use sqlite::{Bindable, Statement};
pub use user::{RequestUser, User, UserByGuid, UserQuery};
mod note;
pub use note::{Note, NoteQuery, RequestNote};

use crate::store::Query;

pub trait DataObject: Sized + Bindable + std::fmt::Debug + Clone {
    fn from_rows(statement: &mut Statement) -> Vec<Self>;
    fn table_name() -> String;
    fn sql_cols() -> String;
    fn id_col() -> String;
}

#[derive(Debug, Deserialize)]
pub(crate) enum DataType {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "note")]
    Note,
}

pub(crate) trait RequestObject: Sized + Bindable + std::fmt::Debug + Clone {
    fn validate_create(&self) -> Result<(), ValidationError>;
    fn validate_update(&self) -> Result<(), ValidationError>;
    fn sql_cols(&self) -> String;
    fn sql_placeholders(&self) -> String;
    fn id(&self) -> Option<i64>;
}

#[derive(Debug)]
pub(crate) enum ValidationError {
    MissingIdOnUpdate,
    MissingRequiredOnCreate(String),
    IdProvidedOnCreate,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ValidationError::MissingIdOnUpdate => {
                write!(fmt, "id required for updates")
            }
            ValidationError::MissingRequiredOnCreate(ref s) => {
                write!(fmt, "missing required field `{}`", s)
            }
            ValidationError::IdProvidedOnCreate => {
                write!(fmt, "id must not be provided for create")
            }
        }
    }
}

impl std::error::Error for ValidationError {
    fn description(&self) -> &str {
        match *self {
            ValidationError::MissingIdOnUpdate => "Missing id error",
            ValidationError::MissingRequiredOnCreate(_) => "Missing required field error",
            ValidationError::IdProvidedOnCreate => "Id provided on create error",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            ValidationError::MissingIdOnUpdate => None,
            ValidationError::MissingRequiredOnCreate(_) => None,
            ValidationError::IdProvidedOnCreate => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum QueryTypes {
    UserQuery(UserQuery),
    NoteQuery(NoteQuery),
}

impl Query for QueryTypes {
    fn build(&self) -> (String, Vec<sqlite::Value>) {
        match self {
            Self::UserQuery(inner) => inner.build(),
            Self::NoteQuery(inner) => inner.build(),
        }
    }
}

impl TryFrom<(&DataType, (&String, &String))> for QueryTypes {
    type Error = ();

    fn try_from((dt, (query, val)): (&DataType, (&String, &String))) -> Result<Self, Self::Error> {
        match dt {
            DataType::User => {
                let uq = UserQuery::try_from((query, val))?;
                Ok(QueryTypes::UserQuery(uq))
            }
            DataType::Note => {
                let nq = NoteQuery::try_from((query, val))?;
                Ok(QueryTypes::NoteQuery(nq))
            }
        }
    }
}
