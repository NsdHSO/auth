use sea_orm_migration::prelude::*;
use ::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Create enum types for system
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DO $$ 
            BEGIN 
                IF NOT EXISTS (
                    SELECT 1 FROM pg_type t
                    JOIN pg_namespace n ON n.oid = t.typnamespace
                    WHERE t.typname = 'user_role' AND n.nspname = 'public'
                ) THEN
                    CREATE TYPE public.user_role AS ENUM (
                        'ADMIN', 'USER', 'MODERATOR', 'GUEST'
                    );
                END IF;
            END $$;"#,
        )).await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM pg_type t
                    JOIN pg_namespace n ON n.oid = t.typnamespace
                    WHERE t.typname = 'user_status' AND n.nspname = 'public'
                ) THEN
                    CREATE TYPE public.user_status AS ENUM (
                        'ACTIVE', 'INACTIVE', 'SUSPENDED', 'PENDING_VERIFICATION'
                    );
                END IF;
            END $$;"#,
        )).await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM pg_type t
                    JOIN pg_namespace n ON n.oid = t.typnamespace
                    WHERE t.typname = 'token_type' AND n.nspname = 'public'
                ) THEN
                    CREATE TYPE public.token_type AS ENUM (
                        'ACCESS', 'REFRESH', 'RESET_PASSWORD', 'EMAIL_VERIFICATION'
                    );
                END IF;
            END $$;"#,
        )).await?;

        // Ensure types in public are resolvable alongside auth
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "SET search_path TO auth, public;",
        )).await?;

        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()"))
                    )
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::FirstName).string())
                    .col(ColumnDef::new(Users::LastName).string())
                    .col(
                        ColumnDef::new(Users::Role)
                            .custom(Alias::new("user_role"))
                            .not_null()
                            .default("USER")
                    )
                    .col(
                        ColumnDef::new(Users::Status)
                            .custom(Alias::new("user_status"))
                            .not_null()
                            .default("PENDING_VERIFICATION")
                    )
                    .col(ColumnDef::new(Users::EmailVerified).boolean().not_null().default(false))
                    .col(ColumnDef::new(Users::LastLogin).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .to_owned(),
            )
            .await?;

        // Create tokens table
        manager
            .create_table(
                Table::create()
                    .table(Tokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()"))
                    )
                    .col(ColumnDef::new(Tokens::UserId).uuid().not_null())
                    .col(ColumnDef::new(Tokens::Token).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(Tokens::TokenType)
                            .custom(Alias::new("token_type"))
                            .not_null()
                    )
                    .col(ColumnDef::new(Tokens::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Tokens::IsRevoked).boolean().not_null().default(false))
                    .col(
                        ColumnDef::new(Tokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        ColumnDef::new(Tokens::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tokens_user_id")
                            .from(Tokens::Table, Tokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create sessions table
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sessions::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()"))
                    )
                    .col(ColumnDef::new(Sessions::UserId).uuid().not_null())
                    .col(ColumnDef::new(Sessions::SessionToken).string().not_null().unique_key())
                    .col(ColumnDef::new(Sessions::IpAddress).string())
                    .col(ColumnDef::new(Sessions::UserAgent).text())
                    .col(ColumnDef::new(Sessions::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Sessions::IsActive).boolean().not_null().default(true))
                    .col(
                        ColumnDef::new(Sessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        ColumnDef::new(Sessions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sessions_user_id")
                            .from(Sessions::Table, Sessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_users_email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_username")
                    .table(Users::Table)
                    .col(Users::Username)
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tokens_user_id")
                    .table(Tokens::Table)
                    .col(Tokens::UserId)
                    .if_not_exists()
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tokens_token")
                    .table(Tokens::Table)
                    .col(Tokens::Token)
                    .if_not_exists()
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sessions_user_id")
                    .table(Sessions::Table)
                    .col(Sessions::UserId)
                    .if_not_exists()
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sessions_token")
                    .table(Sessions::Table)
                    .col(Sessions::SessionToken)
                    .if_not_exists()
                    .to_owned()
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Tokens::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        // Drop enum types
        let db = manager.get_connection();
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DROP TYPE IF EXISTS public.user_role;",
        )).await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DROP TYPE IF EXISTS public.user_status;",
        )).await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DROP TYPE IF EXISTS public.token_type;",
        )).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    Username,
    PasswordHash,
    FirstName,
    LastName,
    Role,
    Status,
    EmailVerified,
    LastLogin,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tokens {
    Table,
    Id,
    UserId,
    Token,
    TokenType,
    ExpiresAt,
    IsRevoked,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    Id,
    UserId,
    SessionToken,
    IpAddress,
    UserAgent,
    ExpiresAt,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
