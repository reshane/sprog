use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Punch {
    pub id: i64,
    pub owner_id: i64,
    pub geo: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestPunch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo: Option<String>,
}

#[cfg(feature = "full")]
pub use ext::*;

#[cfg(feature = "full")]
mod ext {
    use super::{Punch, RequestPunch};
    use sqlite::{Bindable, BindableWithIndex, State, Value};
    use tracing::error;
    use lib_glonk::types::{Criteria, DataObject, EqualsCriteria, Query, RequestObject, ValidationError};
    impl Bindable for Punch {
        fn bind(self, statement: &mut sqlite::Statement) -> sqlite::Result<()> {
            self.id.clone().bind(statement, 1)?;
            self.owner_id.clone().bind(statement, 2)?;
            self.geo.clone().bind(statement, 2)?;
            Ok(())
        }
    }

    impl DataObject for Punch {
        fn from_rows(statement: &mut sqlite::Statement) -> Vec<Self> {
            let mut res = vec![];
            while let Ok(State::Row) = statement.next() {
                res.push(Self {
                    id: statement.read::<i64, _>("id").unwrap(),
                    owner_id: statement.read::<i64, _>("owner_id").unwrap(),
                    geo: statement.read::<String, _>("geo").unwrap(),
                });
            }
            return res;
        }

        fn table_name() -> String {
            "punches".to_string()
        }

        fn sql_cols() -> String {
            "id,owner_id,geo".to_string()
        }

        fn id_col() -> String {
            "id".to_string()
        }

        fn owner_id_col() -> String {
            "owner_id".to_string()
        }
    }

    impl Bindable for RequestPunch {
        fn bind(self, statement: &mut sqlite::Statement) -> sqlite::Result<()> {
            let mut idx = 1;
            if let Some(id) = self.id {
                id.clone().bind(statement, idx)?;
                idx += 1;
            }
            if let Some(owner_id) = self.owner_id {
                owner_id.clone().bind(statement, idx)?;
                idx += 1;
            }
            if let Some(geo) = self.geo {
                geo.clone().as_str().bind(statement, idx)?;
            }
            Ok(())
        }
    }

    impl RequestObject for RequestPunch {
        fn validate_create(&self, owner_id: Option<i64>) -> Result<(), ValidationError> {
            match self.owner_id {
                Some(request_data_owner_id) => {
                    match owner_id {
                        Some(owner_id) if owner_id != request_data_owner_id => {
                            return Err(ValidationError::InvalidOwnerId(format!(
                                "request header owner_id ({}) does not match data owner_id ({})",
                                request_data_owner_id, owner_id
                            )));
                        },
                        Some(_) | None => {},
                    }
                }
                None => {
                    return Err(ValidationError::MissingRequiredOnCreate(String::from(
                        "owner_id",
                    )));
                }
            }
            match self.geo {
                Some(_) => {}
                None => {
                    return Err(ValidationError::MissingRequiredOnCreate(String::from(
                        "geo",
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

        fn validate_update(&self, owner_id: Option<i64>) -> Result<(), ValidationError> {
            match self.owner_id {
                Some(request_data_owner_id) => {
                    match owner_id {
                        Some(owner_id) if owner_id != request_data_owner_id => {
                            return Err(ValidationError::InvalidOwnerId(format!(
                                "request header owner_id ({}) does not match data owner_id ({})",
                                request_data_owner_id, owner_id
                            )));
                        },
                        Some(_) | None => {},
                    }
                }
                None => {
                    return Err(ValidationError::MissingRequiredOnCreate(String::from(
                        "owner_id",
                    )));
                }
            }
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
            if let Some(_) = self.owner_id {
                cols.push("owner_id");
            }
            if let Some(_) = self.geo {
                cols.push("geo");
            }
            cols.join(",")
        }

        fn sql_placeholders(&self) -> String {
            let mut ct = 0;
            if let Some(_) = self.id {
                ct += 1;
            }
            if let Some(_) = self.owner_id {
                ct += 1;
            }
            if let Some(_) = self.geo {
                ct += 1;
            }
            vec!["?"; ct].join(",")
        }

        fn id(&self) -> Option<i64> {
            self.id
        }

        fn owner_id(&self) -> Option<i64> {
            self.owner_id
        }
    }

    // Query types
    #[derive(Debug)]
    pub enum PunchQuery {
        ByOwnerId(PunchByOwnerId),
    }

    impl Query for PunchQuery {
        fn build(&self) -> (String, Vec<sqlite::Value>) {
            match self {
                PunchQuery::ByOwnerId(inner) => inner.build(),
            }
        }
    }

    impl TryFrom<(&String, &String)> for PunchQuery {
        type Error = ();

        fn try_from((q, v): (&String, &String)) -> Result<Self, Self::Error> {
            let q = q.as_str();
            match q {
                "byOwnerId" => {
                    let id = match v.parse::<i64>() {
                        Ok(id) => id,
                        Err(e) => {
                            error!("{:?}", e);
                            return Err(());
                        },
                    };
                    Ok(Self::ByOwnerId(PunchByOwnerId::new(id)))
                },
                _ => {
                    error!("Unrecognized query for Note: {:?}", (q,v));
                    Err(())
                }
            }
        }
    }

    #[derive(Debug)]
    pub struct PunchByOwnerId {
        inner: EqualsCriteria,
    }

    impl PunchByOwnerId {
        pub fn new(val: i64) -> Self {
            Self {
                inner: EqualsCriteria {
                    field: String::from("owner_id"),
                    val: Value::Integer(val),
                }
            }
        }
    }

    impl Query for PunchByOwnerId {
        fn build(&self) -> (String, Vec<sqlite::Value>) {
            self.inner.build()
        }
    }
}
