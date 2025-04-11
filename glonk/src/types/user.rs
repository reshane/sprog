use crate::store::{EqualsCriteria, Query};

use super::{DataObject, RequestObject, ValidationError};
use serde::{Deserialize, Serialize};
use sqlite::{Bindable, BindableWithIndex, State, Statement};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub guid: String,
    pub name: String,
    pub email: String,
    pub picture: String,
}

impl Bindable for User {
    fn bind(self, statement: &mut Statement) -> sqlite::Result<()> {
        self.id.clone().bind(statement, 1)?;
        self.guid.clone().as_str().bind(statement, 2)?;
        self.name.clone().as_str().bind(statement, 3)?;
        self.email.clone().as_str().bind(statement, 4)?;
        self.picture.clone().as_str().bind(statement, 5)?;
        Ok(())
    }
}

impl DataObject for User {
    fn from_rows(statement: &mut Statement) -> Vec<Self> {
        let mut res = vec![];
        while let Ok(State::Row) = statement.next() {
            res.push(Self {
                id: statement.read::<i64, _>("id").unwrap(),
                guid: statement.read::<String, _>("guid").unwrap(),
                name: statement.read::<String, _>("name").unwrap(),
                email: statement.read::<String, _>("email").unwrap(),
                picture: statement.read::<String, _>("picture").unwrap(),
            });
        }
        return res;
    }

    fn table_name() -> String {
        "users".to_string()
    }

    fn sql_cols() -> String {
        "id,guid,name,email,picture".to_string()
    }

    fn id_col() -> String {
        "id".to_string()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestUser {
    pub id: Option<i64>,
    pub guid: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub picture: Option<String>,
}

impl Bindable for RequestUser {
    fn bind(self, statement: &mut Statement) -> sqlite::Result<()> {
        let mut idx = 1;
        if let Some(id) = self.id {
            id.clone().bind(statement, idx)?;
            idx += 1;
        }
        if let Some(guid) = self.guid {
            guid.clone().as_str().bind(statement, idx)?;
            idx += 1;
        }
        if let Some(name) = self.name {
            name.clone().as_str().bind(statement, idx)?;
            idx += 1;
        }
        if let Some(email) = self.email {
            email.clone().as_str().bind(statement, idx)?;
            idx += 1;
        }
        if let Some(picture) = self.picture {
            picture.clone().as_str().bind(statement, idx)?;
        }
        Ok(())
    }
}

impl RequestObject for RequestUser {
    fn validate_create(&self) -> Result<(), ValidationError> {
        match self.guid {
            Some(_) => {}
            None => {
                return Err(ValidationError::MissingRequiredOnCreate(String::from(
                    "guid",
                )));
            }
        }
        match self.name {
            Some(_) => {}
            None => {
                return Err(ValidationError::MissingRequiredOnCreate(String::from(
                    "name",
                )));
            }
        }
        match self.email {
            Some(_) => {}
            None => {
                return Err(ValidationError::MissingRequiredOnCreate(String::from(
                    "email",
                )));
            }
        }
        match self.picture {
            Some(_) => {}
            None => {
                return Err(ValidationError::MissingRequiredOnCreate(String::from(
                    "picture",
                )));
            }
        }
        match self.id {
            Some(_) => {
                return Err(ValidationError::IdProvidedOnCreate);
            }
            None => {}
        }
        Ok(())
    }

    fn validate_update(&self) -> Result<(), ValidationError> {
        match self.id {
            Some(_) => Ok(()),
            None => Err(ValidationError::MissingIdOnUpdate),
        }
    }

    fn sql_cols(&self) -> String {
        let mut cols = vec![];
        if let Some(_) = self.id {
            cols.push("id");
        }
        if let Some(_) = self.guid {
            cols.push("guid");
        }
        if let Some(_) = self.name {
            cols.push("name");
        }
        if let Some(_) = self.email {
            cols.push("email");
        }
        if let Some(_) = self.picture {
            cols.push("picture");
        }
        cols.join(",")
    }

    fn sql_placeholders(&self) -> String {
        let mut ct = 0;
        if let Some(_) = self.id {
            ct += 1;
        }
        if let Some(_) = self.guid {
            ct += 1;
        }
        if let Some(_) = self.name {
            ct += 1;
        }
        if let Some(_) = self.email {
            ct += 1;
        }
        if let Some(_) = self.picture {
            ct += 1;
        }
        vec!["?"; ct].join(",")
    }

    fn id(&self) -> Option<i64> {
        self.id
    }
}

// Query types
#[derive(Debug)]
pub enum UserQuery {
    ByGuid(UserByGuid),
}

impl Query for UserQuery {
    fn build(&self) -> (String, Vec<sqlite::Value>) {
        match self {
            UserQuery::ByGuid(inner) => inner.build(),
        }
    }
}

impl TryFrom<(&String, &String)> for UserQuery {
    type Error = ();

    fn try_from((q, v): (&String, &String)) -> Result<Self, Self::Error> {
        let q = q.as_str();
        match q {
            "byGuid" => Ok(Self::ByGuid(UserByGuid::new(v.to_string()))),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct UserByGuid {
    inner: EqualsCriteria,
}

impl UserByGuid {
    pub fn new(val: String) -> Self {
        Self {
            inner: EqualsCriteria {
                field: String::from("guid"),
                val: sqlite::Value::String(val),
            },
        }
    }
}

impl Query for UserByGuid {
    fn build(&self) -> (String, Vec<sqlite::Value>) {
        use crate::store::Criteria;
        self.inner.build()
    }
}
