use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub owner_id: i64,
    pub note_id: i64,
    pub contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestComment {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contents: Option<String>,
}

#[cfg(feature = "full")]
pub use ext::*;

#[cfg(feature = "full")]
mod ext {
    use super::{Comment, RequestComment};
    use lib_glonk::types::{
        ContainsCriteria, Criteria, DataObject, EqualsCriteria, Query, RequestObject,
        ValidationError,
    };
    use sqlite::{Bindable, BindableWithIndex, State, Value};
    use tracing::error;

    impl Bindable for Comment {
        fn bind(self, statement: &mut sqlite::Statement) -> sqlite::Result<()> {
            self.id.clone().bind(statement, 1)?;
            self.owner_id.clone().bind(statement, 2)?;
            self.note_id.clone().bind(statement, 3)?;
            self.contents.clone().as_str().bind(statement, 4)?;
            Ok(())
        }
    }
    impl DataObject for Comment {
        fn from_rows(statement: &mut sqlite::Statement) -> Vec<Self> {
            let mut res = vec![];
            while let Ok(State::Row) = statement.next() {
                res.push(Self {
                    id: statement.read::<i64, _>("id").unwrap(),
                    owner_id: statement.read::<i64, _>("owner_id").unwrap(),
                    note_id: statement.read::<i64, _>("note_id").unwrap(),
                    contents: statement.read::<String, _>("contents").unwrap(),
                });
            }
            return res;
        }

        fn table_name() -> String {
            "comments".to_string()
        }

        fn sql_cols() -> String {
            "id,owner_id,note_id,contents".to_string()
        }

        fn id_col() -> String {
            "id".to_string()
        }

        fn owner_id_col() -> String {
            "owner_id".to_string()
        }
    }

    impl Bindable for RequestComment {
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
            if let Some(note_id) = self.note_id {
                note_id.clone().bind(statement, idx)?;
                idx += 1;
            }
            if let Some(contents) = self.contents {
                contents.clone().as_str().bind(statement, idx)?;
            }
            Ok(())
        }
    }
    impl RequestObject for RequestComment {
        fn validate_create(
            &self,
            owner_id: Option<i64>,
        ) -> Result<(), lib_glonk::types::ValidationError> {
            match self.owner_id {
                Some(request_data_owner_id) => match owner_id {
                    Some(owner_id) if owner_id != request_data_owner_id => {
                        return Err(ValidationError::InvalidOwnerId(format!(
                            "request header owner_id ({}) does not match data owner_id ({})",
                            request_data_owner_id, owner_id
                        )));
                    }
                    Some(_) | None => {}
                },
                None => {
                    return Err(ValidationError::MissingRequiredOnCreate(String::from(
                        "owner_id",
                    )));
                }
            }
            match self.contents {
                Some(_) => {}
                None => {
                    return Err(ValidationError::MissingRequiredOnCreate(String::from(
                        "contents",
                    )));
                }
            }
            match self.note_id {
                Some(_) => {}
                None => {
                    return Err(ValidationError::MissingRequiredOnCreate(String::from(
                        "note_id",
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

        fn validate_update(
            &self,
            owner_id: Option<i64>,
        ) -> Result<(), lib_glonk::types::ValidationError> {
            match self.owner_id {
                Some(request_data_owner_id) => match owner_id {
                    Some(owner_id) if owner_id != request_data_owner_id => {
                        return Err(ValidationError::InvalidOwnerId(format!(
                            "request header owner_id ({}) does not match data owner_id ({})",
                            request_data_owner_id, owner_id
                        )));
                    }
                    Some(_) | None => {}
                },
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
            if let Some(_) = self.note_id {
                cols.push("note_id");
            }
            if let Some(_) = self.contents {
                cols.push("contents");
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
            if let Some(_) = self.note_id {
                ct += 1;
            }
            if let Some(_) = self.contents {
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
    pub enum CommentQuery {
        ByContentsContains(CommentContentsContains),
        ByOwnerId(CommentByOwnerId),
        ByNoteId(CommentByNoteId),
    }

    impl Query for CommentQuery {
        fn build(&self) -> (String, Vec<sqlite::Value>) {
            match self {
                CommentQuery::ByContentsContains(inner) => inner.build(),
                CommentQuery::ByOwnerId(inner) => inner.build(),
                CommentQuery::ByNoteId(inner) => inner.build(),
            }
        }
    }

    impl TryFrom<(&String, &String)> for CommentQuery {
        type Error = ();

        fn try_from((q, v): (&String, &String)) -> Result<Self, Self::Error> {
            let q = q.as_str();
            match q {
                "byContentsContains" => Ok(Self::ByContentsContains(CommentContentsContains::new(
                    v.to_string(),
                ))),
                "byOwnerId" => {
                    let id = match v.parse::<i64>() {
                        Ok(id) => id,
                        Err(e) => {
                            error!("{:?}", e);
                            return Err(());
                        }
                    };
                    Ok(Self::ByOwnerId(CommentByOwnerId::new(id)))
                }
                "byNoteId" => {
                    let id = match v.parse::<i64>() {
                        Ok(id) => id,
                        Err(e) => {
                            error!("{:?}", e);
                            return Err(());
                        }
                    };
                    Ok(Self::ByNoteId(CommentByNoteId::new(id)))
                }
                _ => {
                    error!("Unrecognized query for Comment: {:?}", (q, v));
                    Err(())
                }
            }
        }
    }

    #[derive(Debug)]
    pub struct CommentByNoteId {
        inner: EqualsCriteria,
    }

    impl CommentByNoteId {
        fn new(val: i64) -> Self {
            Self {
                inner: EqualsCriteria {
                    field: String::from("note_id"),
                    val: Value::Integer(val),
                },
            }
        }
    }

    impl Query for CommentByNoteId {
        fn build(&self) -> (String, Vec<Value>) {
            self.inner.build()
        }
    }

    #[derive(Debug)]
    pub struct CommentByOwnerId {
        inner: EqualsCriteria,
    }

    impl CommentByOwnerId {
        pub fn new(val: i64) -> Self {
            Self {
                inner: EqualsCriteria {
                    field: String::from("owner_id"),
                    val: Value::Integer(val),
                },
            }
        }
    }

    impl Query for CommentByOwnerId {
        fn build(&self) -> (String, Vec<sqlite::Value>) {
            self.inner.build()
        }
    }

    #[derive(Debug)]
    pub struct CommentContentsContains {
        inner: ContainsCriteria,
    }

    impl CommentContentsContains {
        pub fn new(val: String) -> Self {
            Self {
                inner: ContainsCriteria {
                    field: String::from("contents"),
                    val,
                },
            }
        }
    }

    impl Query for CommentContentsContains {
        fn build(&self) -> (String, Vec<sqlite::Value>) {
            self.inner.build()
        }
    }
}
