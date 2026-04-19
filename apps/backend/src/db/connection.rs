use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

use crate::config::Config;

pub async fn connect(config: &Config) -> anyhow::Result<DatabaseConnection> {
    let db = Database::connect(&config.database_url).await?;

    if config.run_migrations_on_startup {
        println!("Running migrations...");
        Migrator::up(&db, None).await?;
        println!("Migrations completed.");
    }

    Ok(db)
}
