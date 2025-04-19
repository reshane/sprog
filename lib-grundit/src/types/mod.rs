mod user;
mod note;
mod comment;
mod punch;

pub use user::User;
pub use note::Note;
pub use comment::Comment;
pub use punch::{RequestPunch, Punch};

#[cfg(feature = "full")]
pub use ext::*;

#[cfg(feature = "full")]
mod ext {
    pub use super::user::{RequestUser, UserByGuid, UserQuery};
    pub use super::note::{RequestNote, NoteQuery};
    pub use super::comment::{RequestComment, CommentQuery};
    pub use super::punch::PunchQuery;

    use serde::Deserialize;
    use axum::{extract::{Query as UrlQuery, rejection::{PathRejection, QueryRejection}, FromRequestParts, Path}, http::request::Parts, response::IntoResponse};
    use std::collections::HashMap;
    use tracing::debug;
    use lib_glonk::types::Query;

    // Application specific
    #[derive(Debug, Deserialize)]
    pub enum DataType {
        #[serde(rename = "user")]
        User,
        #[serde(rename = "note")]
        Note,
        #[serde(rename = "comment")]
        Comment,
        #[serde(rename = "punch")]
        Punch,
    }

    #[derive(Debug)]
    pub(crate) enum QueryTypes {
        UserQuery(UserQuery),
        NoteQuery(NoteQuery),
        CommentQuery(CommentQuery),
        PunchQuery(PunchQuery),
    }

    impl Query for QueryTypes {
        fn build(&self) -> (String, Vec<sqlite::Value>) {
            match self {
                Self::UserQuery(inner) => inner.build(),
                Self::NoteQuery(inner) => inner.build(),
                Self::CommentQuery(inner) => inner.build(),
                Self::PunchQuery(inner) => inner.build(),
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
                DataType::Comment => {
                    let nq = CommentQuery::try_from((query, val))?;
                    Ok(QueryTypes::CommentQuery(nq))
                }
                DataType::Punch => {
                    let nq = PunchQuery::try_from((query, val))?;
                    Ok(QueryTypes::PunchQuery(nq))
                }
            }
        }
    }

    pub struct ExtractGlonkQueries(pub Vec<Box<dyn Query>>);

    pub enum QueriesRejection {
        Query(QueryRejection),
        Path(PathRejection),
    }

    impl IntoResponse for QueriesRejection {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Query(i) => i.into_response(),
                Self::Path(i) => i.into_response(),
            }
        }
    }

    impl From<PathRejection> for QueriesRejection {
        fn from(value: PathRejection) -> QueriesRejection {
            QueriesRejection::Path(value)
        }
    }
    impl From<QueryRejection> for QueriesRejection {
        fn from(value: QueryRejection) -> Self {
            QueriesRejection::Query(value)
        }
    }

    impl From<QueryTypes> for Box<dyn lib_glonk::types::Query> {
        fn from(value: QueryTypes) -> Self {
            Box::new(value)
        }
    }

    impl<S> FromRequestParts<S> for ExtractGlonkQueries
    where
        S: Send + Sync,
    {
        type Rejection = QueriesRejection;

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let queries = UrlQuery::<HashMap<String, String>>::from_request_parts(parts, state).await?;
            let Path(data_type) = Path::from_request_parts(parts, state).await?;
            let glonk_queries = queries
                .iter()
                .filter_map(|(k, v)| QueryTypes::try_from((&data_type, (k, v))).ok())
                .map(|qt| {
                    qt.into()
                })
                .collect::<Vec<Box<dyn Query>>>();
            debug!("{:?}", queries);
            Ok(Self(glonk_queries))
        }
    }
}
