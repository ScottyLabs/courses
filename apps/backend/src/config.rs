use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub hostname: String,
    pub port: u16,
    pub run_migrations_on_startup: bool,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let hostname = env::var("BACKEND_HOSTNAME").unwrap_or("127.0.0.1".to_string());
        let port = env::var("BACKEND_PORT")
            .unwrap_or("3000".to_string())
            .parse()
            .context("BACKEND_PORT must be a valid u16")?;
        let run_migrations_on_startup = env::var("RUN_MIGRATIONS_ON_STARTUP")
            .unwrap_or("false".to_string())
            .parse()
            .context("RUN_MIGRATIONS_ON_STARTUP must be a boolean")?;
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

        Ok(Self {
            hostname,
            port,
            run_migrations_on_startup,
            database_url,
        })
    }

    pub fn get_host(&self) -> String {
        format!("{}:{}", self.hostname, self.port)
    }
}
