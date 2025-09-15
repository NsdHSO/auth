use sea_orm_migration::prelude::*;
use ::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Ensure schema and search_path
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "CREATE SCHEMA IF NOT EXISTS auth;".to_string(),
        ))
        .await?;
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "SET search_path TO auth, public;".to_string(),
        ))
        .await?;

        // Enable pg_trgm for trigram indexes (safe if already enabled)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "CREATE EXTENSION IF NOT EXISTS pg_trgm;".to_string(),
        ))
        .await?;

        // Add a generated tsvector column for full-text search across username/first_name/last_name/email
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DO $$
            DECLARE
                v_schema text := CASE
                    WHEN EXISTS (
                        SELECT 1 FROM information_schema.tables
                        WHERE table_schema = 'auth' AND table_name = 'users'
                    ) THEN 'auth'
                    WHEN EXISTS (
                        SELECT 1 FROM information_schema.tables
                        WHERE table_schema = 'public' AND table_name = 'users'
                    ) THEN 'public'
                    ELSE NULL
                END;
            BEGIN
                IF v_schema IS NULL THEN
                    RAISE EXCEPTION 'users table not found in auth or public schema';
                END IF;

                IF NOT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = v_schema
                      AND table_name = 'users'
                      AND column_name = 'search_tsv'
                ) THEN
                    EXECUTE format(
                        $sql$ALTER TABLE %I.users
                          ADD COLUMN search_tsv tsvector GENERATED ALWAYS AS (
                            setweight(to_tsvector('simple', coalesce(username, '')), 'A') ||
                            setweight(to_tsvector('simple', coalesce(first_name, '')), 'B') ||
                            setweight(to_tsvector('simple', coalesce(last_name, '')), 'B') ||
                            setweight(to_tsvector('simple', coalesce(email, '')), 'C')
                          ) STORED$sql$,
                        v_schema
                    );
                END IF;
            END $$;
            "#.to_string(),
        ))
        .await?;

        // Create GIN index for the tsvector column
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DO $$
            DECLARE
                v_schema text := CASE
                    WHEN EXISTS (
                        SELECT 1 FROM information_schema.tables
                        WHERE table_schema = 'auth' AND table_name = 'users'
                    ) THEN 'auth'
                    WHEN EXISTS (
                        SELECT 1 FROM information_schema.tables
                        WHERE table_schema = 'public' AND table_name = 'users'
                    ) THEN 'public'
                    ELSE NULL
                END;
            BEGIN
                IF v_schema IS NULL THEN
                    RAISE EXCEPTION 'users table not found in auth or public schema';
                END IF;

                IF NOT EXISTS (
                    SELECT 1 FROM pg_indexes
                    WHERE schemaname = v_schema AND indexname = 'idx_users_search_tsv'
                ) THEN
                    EXECUTE format(
                        $sql$CREATE INDEX idx_users_search_tsv ON %I.users USING gin (search_tsv)$sql$,
                        v_schema
                    );
                END IF;
            END $$;
            "#.to_string(),
        ))
        .await?;

        // Create trigram GIN indexes for fuzzy/substring search on common fields
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DO $$
            DECLARE
                v_schema text := CASE
                    WHEN EXISTS (
                        SELECT 1 FROM information_schema.tables
                        WHERE table_schema = 'auth' AND table_name = 'users'
                    ) THEN 'auth'
                    WHEN EXISTS (
                        SELECT 1 FROM information_schema.tables
                        WHERE table_schema = 'public' AND table_name = 'users'
                    ) THEN 'public'
                    ELSE NULL
                END;
            BEGIN
                IF v_schema IS NULL THEN
                    RAISE EXCEPTION 'users table not found in auth or public schema';
                END IF;

                IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname=v_schema AND indexname='idx_users_username_trgm') THEN
                    EXECUTE format($sql$CREATE INDEX idx_users_username_trgm ON %I.users USING gin (lower(username) gin_trgm_ops)$sql$, v_schema);
                END IF;
                IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname=v_schema AND indexname='idx_users_email_trgm') THEN
                    EXECUTE format($sql$CREATE INDEX idx_users_email_trgm ON %I.users USING gin (lower(email) gin_trgm_ops)$sql$, v_schema);
                END IF;
                IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname=v_schema AND indexname='idx_users_first_name_trgm') THEN
                    EXECUTE format($sql$CREATE INDEX idx_users_first_name_trgm ON %I.users USING gin (lower(first_name) gin_trgm_ops)$sql$, v_schema);
                END IF;
                IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname=v_schema AND indexname='idx_users_last_name_trgm') THEN
                    EXECUTE format($sql$CREATE INDEX idx_users_last_name_trgm ON %I.users USING gin (lower(last_name) gin_trgm_ops)$sql$, v_schema);
                END IF;
            END $$;
            "#.to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "SET search_path TO auth, public;".to_string(),
        ))
        .await?;

        // Drop indexes first, then the column. We do not drop the pg_trgm extension.
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DO $$
            DECLARE
                v_schema text := CASE
                    WHEN EXISTS (
                        SELECT 1 FROM information_schema.tables
                        WHERE table_schema = 'auth' AND table_name = 'users'
                    ) THEN 'auth'
                    WHEN EXISTS (
                        SELECT 1 FROM information_schema.tables
                        WHERE table_schema = 'public' AND table_name = 'users'
                    ) THEN 'public'
                    ELSE NULL
                END;
            BEGIN
                IF v_schema IS NULL THEN
                    RETURN; -- nothing to do
                END IF;

                -- Drop indexes if they exist in the detected schema
                EXECUTE format('DROP INDEX IF EXISTS %I.idx_users_search_tsv', v_schema);
                EXECUTE format('DROP INDEX IF EXISTS %I.idx_users_username_trgm', v_schema);
                EXECUTE format('DROP INDEX IF EXISTS %I.idx_users_email_trgm', v_schema);
                EXECUTE format('DROP INDEX IF EXISTS %I.idx_users_first_name_trgm', v_schema);
                EXECUTE format('DROP INDEX IF EXISTS %I.idx_users_last_name_trgm', v_schema);

                -- Drop column if exists
                EXECUTE format('ALTER TABLE IF EXISTS %I.users DROP COLUMN IF EXISTS search_tsv', v_schema);
            END $$;
            "#.to_string(),
        ))
        .await?;

        Ok(())
    }
}
