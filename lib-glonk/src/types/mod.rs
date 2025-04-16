use sqlite::{Bindable, Statement, Value};
use std::fmt;

// framework
pub trait DataObject: Sized + Bindable + std::fmt::Debug + Clone {
    fn from_rows(statement: &mut Statement) -> Vec<Self>;
    fn table_name() -> String;
    fn sql_cols() -> String;
    fn id_col() -> String;
    fn owner_id_col() -> String;
}

pub trait RequestObject: Sized + Bindable + std::fmt::Debug + Clone {
    fn validate_create(&self, owner_id: Option<i64>) -> Result<(), ValidationError>;
    fn validate_update(&self, owner_id: Option<i64>) -> Result<(), ValidationError>;
    fn sql_cols(&self) -> String;
    fn sql_placeholders(&self) -> String;
    fn id(&self) -> Option<i64>;
    fn owner_id(&self) -> Option<i64>;
}

// validation
#[derive(Debug)]
pub enum ValidationError {
    MissingIdOnUpdate,
    MissingRequiredOnCreate(String),
    InvalidOwnerId(String),
    IdProvidedOnCreate,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ValidationError::MissingIdOnUpdate => {
                write!(fmt, "id required for updates")
            },
            ValidationError::MissingRequiredOnCreate(ref s) => {
                write!(fmt, "missing required field `{}`", s)
            },
            ValidationError::InvalidOwnerId(ref s) => {
                write!(fmt, "{}", s)
            },
            ValidationError::IdProvidedOnCreate => {
                write!(fmt, "id must not be provided for create")
            },
        }
    }
}

impl std::error::Error for ValidationError {
    fn description(&self) -> &str {
        match *self {
            ValidationError::MissingIdOnUpdate => "Missing id error",
            ValidationError::MissingRequiredOnCreate(_) => "Missing required field error",
            ValidationError::InvalidOwnerId(_) => "Invalid owner_id in request error",
            ValidationError::IdProvidedOnCreate => "Id provided on create error",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            ValidationError::MissingIdOnUpdate => None,
            ValidationError::MissingRequiredOnCreate(_) => None,
            ValidationError::InvalidOwnerId(_) => None,
            ValidationError::IdProvidedOnCreate => None,
        }
    }
}

// queries
pub trait Criteria: Send + Sync + std::fmt::Debug {
    fn build(&self) -> (String, Vec<Value>);
}

pub trait Query: Send + Sync + std::fmt::Debug {
    fn build(&self) -> (String, Vec<Value>);
}

#[derive(Debug)]
pub struct ContainsCriteria {
    pub field: String,
    pub val: String,
}

impl Criteria for ContainsCriteria {
    fn build(&self) -> (String, Vec<Value>) {
        (
            format!("{} LIKE ?", self.field),
            vec![Value::String(format!("%{}%", self.val))],
        )
    }
}

#[derive(Debug)]
pub struct EqualsCriteria {
    pub field: String,
    pub val: Value,
}

impl Criteria for EqualsCriteria {
    fn build(&self) -> (String, Vec<Value>) {
        (format!("{} = ?", self.field), vec![self.val.clone()])
    }
}

#[derive(Debug)]
pub struct AndCriteria<L, R>
where
    L: Criteria,
    R: Criteria,
{
    pub left: L,
    pub right: R,
}

impl<L, R> Criteria for AndCriteria<L, R>
where
    L: Criteria,
    R: Criteria,
{
    fn build(&self) -> (String, Vec<Value>) {
        let (lq, lv) = self.left.build();
        let (rq, rv) = self.right.build();
        (
            format!("(({}) and ({}))", lq, rq),
            [&lv[..], &rv[..]].concat(),
        )
    }
}

#[derive(Debug)]
pub struct OrCriteria<L, R>
where
    L: Criteria,
    R: Criteria,
{
    pub left: L,
    pub right: R,
}

impl<L, R> Criteria for OrCriteria<L, R>
where
    L: Criteria,
    R: Criteria,
{
    fn build(&self) -> (String, Vec<Value>) {
        let (lq, lv) = self.left.build();
        let (rq, rv) = self.right.build();
        (
            format!("(({}) or ({}))", lq, rq),
            [&lv[..], &rv[..]].concat(),
        )
    }
}
