use sea_orm::Database;
use sea_orm_migration::prelude::*;
use std::error::Error;

mod migrator;

const DB_URL: &str = "sqlite:./sqlite.db?mode=rwc";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db = Database::connect(DB_URL).await?;

    let schema_manager = SchemaManager::new(&db);

    migrator::Migrator::refresh(&db).await?;
    assert!(schema_manager.has_table("report").await?);

    Ok(())
}
