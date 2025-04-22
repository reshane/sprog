pub mod error;
pub mod sqlitestore;
use error::StoreResult;
pub use sqlitestore::SqliteStore;

use crate::types::{DataObject, Query, RequestObject};

pub trait Store {
    fn create<R: RequestObject, T: DataObject>(&self, data: R) -> StoreResult<T>;
    fn update<R: RequestObject, T: DataObject>(&self, data: R) -> StoreResult<T>;
    fn get<T: DataObject>(&self, id: i64) -> Option<T>;
    fn get_queries<T: DataObject>(&self, queries: Vec<Box<dyn Query>>) -> Vec<T>;
    fn delete<T: DataObject>(&self, id: i64, owner_id: Option<i64>) -> StoreResult<T>;
}
