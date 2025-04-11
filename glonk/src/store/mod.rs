pub(crate) mod error;
pub(crate) mod sqlitestore;
use std::collections::HashMap;

use axum::{
    extract::{
        FromRequestParts, Path, Query as UrlQuery,
        rejection::{PathRejection, QueryRejection},
    },
    http::request::Parts,
    response::IntoResponse,
};
pub use sqlitestore::SqliteStore;

use error::StoreResult;
use sqlite::Value;
use tracing::debug;

use crate::types::{DataObject, QueryTypes, RequestObject};

pub(crate) trait Store {
    fn create<R: RequestObject, T: DataObject>(&self, data: R) -> StoreResult<T>;
    fn update<R: RequestObject, T: DataObject>(&self, data: R) -> StoreResult<T>;
    fn get<T: DataObject>(&self, id: i64) -> Option<T>;
    fn get_queries<T: DataObject>(&self, queries: Vec<QueryTypes>) -> Vec<T>;
    fn delete<T: DataObject>(&self, id: i64) -> StoreResult<T>;
}

pub struct ExtractGlonkQueries(pub Vec<QueryTypes>);

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
            .collect::<Vec<QueryTypes>>();
        debug!("{:?}", queries);
        Ok(Self(glonk_queries))
    }
}

pub(crate) trait Criteria: Send + Sync + std::fmt::Debug {
    fn build(&self) -> (String, Vec<Value>);
}

pub(crate) trait Query: Send + Sync + std::fmt::Debug {
    fn build(&self) -> (String, Vec<Value>);
}

#[derive(Debug)]
pub(crate) struct ContainsCriteria {
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
pub(crate) struct EqualsCriteria {
    pub field: String,
    pub val: Value,
}

impl Criteria for EqualsCriteria {
    fn build(&self) -> (String, Vec<Value>) {
        (format!("{} = ?", self.field), vec![self.val.clone()])
    }
}

#[derive(Debug)]
pub(crate) struct AndCriteria<L, R>
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
pub(crate) struct OrCriteria<L, R>
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
