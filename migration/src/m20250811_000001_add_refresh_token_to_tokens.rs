use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add nullable refresh_token column to auth.tokens
        manager
            .alter_table(
                Table::alter()
                    .table(TableRef::SchemaTable(Alias::new("auth").into_iden(), Alias::new("tokens").into_iden()))
                    .add_column(
                        ColumnDef::new(Tokens::RefreshToken)
                            .string() // VARCHAR
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove refresh_token column from auth.tokens
        manager
            .alter_table(
                Table::alter()
                    .table(TableRef::SchemaTable(Alias::new("auth").into_iden(), Alias::new("tokens").into_iden()))
                    .drop_column(Tokens::RefreshToken)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Tokens {
    Table,
    RefreshToken,
}

