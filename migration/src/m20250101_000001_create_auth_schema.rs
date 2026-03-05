use sea_orm_migration::prelude::*;
use ::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Create auth schema - this must happen before SeaORM tries to use it
        println!("Creating auth schema...");
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "CREATE SCHEMA IF NOT EXISTS auth;".to_string(),
        )).await?;
        println!("Auth schema created successfully");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop auth schema (will fail if it has objects in it, which is safe)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DROP SCHEMA IF EXISTS auth CASCADE;".to_string(),
        )).await?;

        Ok(())
    }
}
