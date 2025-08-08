use crate::sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add login_history column
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .add_column(
                        ColumnDef::new(Alias::new("login_history"))
                            .json_binary()
                            .null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .to_owned(),
            )
            .await?;

        // Create GIN index using raw SQL by executing on the connection
        manager
            .get_connection()
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres, // Use sea_orm::DatabaseBackend
                r#"
                CREATE INDEX idx_users_login_history
                ON users
                USING GIN (login_history)
                "#.to_string(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the index first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_login_history")
                    .table(Alias::new("users"))
                    .to_owned(),
            )
            .await?;

        // Then drop the column
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .drop_column(Alias::new("login_history"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
