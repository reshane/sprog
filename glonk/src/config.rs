pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}

#[derive(Debug)]
pub struct DbConfig {
    pub host: String,
    pub user: String,
    pub pass: String,
}

impl DbConfig {
    pub fn get_connection_string(&self) -> String {
        format!("postgres://{}:{}@{}/mydb", self.user, self.pass, self.host)
    }
}

#[derive(Debug)]
pub struct Configuration {
    pub port: String,
    pub db: DbConfig,
}

impl Configuration {
    pub fn from_env() -> Result<Self> {
        let db_host = std::env::var("DB_HOST").expect("DB_HOST env var required but not found");
        let db_user = std::env::var("DB_USER").expect("DB_USER env var required but not found");
        let db_pass = std::env::var("DB_PASS").expect("DB_PASS env var required but not found");

        Ok(Configuration {
            port: std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            db: DbConfig {
                host: db_host,
                user: db_user,
                pass: db_pass,
            },
        })
    }
}
