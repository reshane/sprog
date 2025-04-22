use sqlite::{Connection, Value};
use std::sync::Mutex;
use tracing::debug;

use crate::types::{DataObject, Query, RequestObject};

use super::{Store, error::StoreResult};

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn new() -> Self {
        let connection = sqlite::open("test.db").unwrap();
        Self {
            conn: Mutex::new(connection),
        }
    }
}

impl Store for SqliteStore {
    fn create<R: RequestObject, T: DataObject>(&self, data: R) -> StoreResult<T> {
        let query = format!(
            "INSERT INTO {}({}) VALUES ({}) returning {}",
            T::table_name(),
            data.sql_cols(),
            data.sql_placeholders(),
            T::sql_cols()
        );
        debug!("{}", query);
        if let Ok(conn) = self.conn.lock() {
            let mut statement = conn.prepare(query).unwrap();
            statement.bind(data).unwrap();
            let data: Vec<T> = T::from_rows(&mut statement);
            if data.len() >= 1 {
                Ok(data[0].clone())
            } else {
                Err(super::error::StoreError::NotCreated)
            }
        } else {
            Err(super::error::StoreError::NotCreated)
        }
    }

    fn update<R: RequestObject, T: DataObject>(&self, data: R) -> StoreResult<T> {
        let id = match data.id() {
            Some(id) => id,
            None => {
                println!("No id on request onject");
                return Err(crate::store::error::StoreError::NotCreated);
            }
        };
        let owner_id = match data.owner_id() {
            Some(owner_id) => owner_id,
            None => {
                println!("No owner_id on request onject");
                return Err(crate::store::error::StoreError::NotCreated);
            }
        };
        let query = format!(
            "UPDATE {} SET ({}) = ({}) where ({} = :id and {} = :owner_id) returning {}",
            T::table_name(),
            data.sql_cols(),
            data.sql_placeholders(),
            T::id_col(),
            T::owner_id_col(),
            T::sql_cols()
        );
        debug!("{:?}", query);
        if let Ok(conn) = self.conn.lock() {
            let mut statement = conn.prepare(query).unwrap();
            statement.bind(data).unwrap();
            statement.bind((":id", id)).unwrap();
            statement.bind((":owner_id", owner_id)).unwrap();
            let data: Vec<T> = T::from_rows(&mut statement);
            if data.len() >= 1 {
                Ok(data[0].clone())
            } else {
                Err(super::error::StoreError::NotCreated)
            }
        } else {
            Err(super::error::StoreError::NotCreated)
        }
    }

    fn get<T: DataObject>(&self, id: i64) -> Option<T> {
        let query = format!(
            "SELECT * FROM {} where ({} = ?)",
            T::table_name(),
            T::id_col()
        );
        if let Ok(conn) = self.conn.lock() {
            let mut statement = conn.prepare(query).unwrap();
            statement.bind((1, id)).unwrap();
            let data: Vec<T> = T::from_rows(&mut statement);
            if data.len() >= 1 {
                Some(data[0].clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_queries<T: DataObject>(&self, queries: Vec<Box<dyn Query>>) -> Vec<T> {
        let mut clauses = vec![];
        let mut bindables = vec![];
        let mut i = 0;
        for q in queries.iter() {
            let (clause, vals) = q.build();
            clauses.push(clause);
            vals.into_iter().for_each(|v| {
                bindables.push((i + 1, v));
                i += 1;
            });
        }
        let mut query = format!("SELECT * FROM {}", T::table_name());
        if clauses.len() > 0 {
            let clauses_str = format!(" where {}", clauses.join(" and "));
            query.push_str(clauses_str.as_str());
        }
        debug!("{} {:?}", query, queries);
        if let Ok(conn) = self.conn.lock() {
            let mut statement = conn.prepare(query).unwrap();
            statement
                .bind::<&[(_, Value)]>(&bindables.as_slice()[..])
                .unwrap();
            let data: Vec<T> = T::from_rows(&mut statement);
            data
        } else {
            vec![]
        }
    }

    fn delete<T: DataObject>(&self, id: i64, owner_id: Option<i64>) -> StoreResult<T> {
        let (clauses, params) = match owner_id {
            Some(owner_id) if T::owner_id_col() == T::id_col() => (
                format!("({} = ? and {} = ?)", T::id_col(), T::owner_id_col()),
                vec![(1, Value::Integer(id)), (2, Value::Integer(owner_id))],
            ),
            Some(_) | None => (
                format!("({} = ?)", T::id_col()),
                vec![(1, Value::Integer(id))],
            ),
        };
        let query = format!(
            "DELETE FROM {} where {} returning {}",
            T::table_name(),
            clauses,
            T::sql_cols()
        );
        if let Ok(conn) = self.conn.lock() {
            let mut statement = conn.prepare(query).unwrap();
            statement
                .bind::<&[(_, Value)]>(&params.as_slice()[..])
                .unwrap();
            let data: Vec<T> = T::from_rows(&mut statement);
            if data.len() >= 1 {
                Ok(data[0].clone())
            } else {
                Err(super::error::StoreError::NotFound)
            }
        } else {
            Err(super::error::StoreError::NotFound)
        }
    }
}
