use sea_orm_migration::prelude::*;
use ::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Ensure we are operating on auth schema first
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "SET search_path TO auth, public;".to_string(),
        ))
        .await?;

        // 1) Backfill base-role mapping wherever missing
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO auth.user_roles (user_id, role_id)
            SELECT u.id, r.id
            FROM auth.users u
            JOIN auth.roles r ON r.code = u.role::text
            LEFT JOIN auth.user_roles ur ON ur.user_id = u.id AND ur.role_id = r.id
            WHERE ur.user_id IS NULL;
            "#.to_string(),
        ))
        .await?;

        // 2) Remove any extra user_roles entries that do not match the user's base role
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DELETE FROM auth.user_roles ur
            USING auth.users u, auth.roles r
            WHERE ur.user_id = u.id
              AND ur.role_id = r.id
              AND r.code <> u.role::text;
            "#.to_string(),
        ))
        .await?;

        // 3) Enforce at-most-one role per user via unique index
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM pg_indexes
                    WHERE schemaname = 'auth' AND indexname = 'ux_user_roles_user_id'
                ) THEN
                    CREATE UNIQUE INDEX ux_user_roles_user_id ON auth.user_roles (user_id);
                END IF;
            END $$;
            "#.to_string(),
        ))
        .await?;

        // 4) Create function to sync base role on INSERT/UPDATE of auth.users
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            CREATE OR REPLACE FUNCTION auth.sync_user_base_role()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $$
            BEGIN
              IF TG_OP = 'INSERT' THEN
                -- Ensure only base mapping exists for new user
                DELETE FROM auth.user_roles WHERE user_id = NEW.id;
                INSERT INTO auth.user_roles (user_id, role_id)
                SELECT NEW.id, r.id
                FROM auth.roles r
                WHERE r.code = NEW.role::text
                ON CONFLICT (user_id, role_id) DO NOTHING;
                RETURN NEW;
              END IF;

              IF TG_OP = 'UPDATE' AND NEW.role <> OLD.role THEN
                -- Replace any existing mappings with only the new base role
                DELETE FROM auth.user_roles WHERE user_id = NEW.id;
                INSERT INTO auth.user_roles (user_id, role_id)
                SELECT NEW.id, r.id
                FROM auth.roles r
                WHERE r.code = NEW.role::text
                ON CONFLICT (user_id, role_id) DO NOTHING;
              END IF;

              RETURN NEW;
            END;
            $$;
            "#.to_string(),
        ))
        .await?;

        // 5) Attach triggers to auth.users (split into single statements)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP TRIGGER IF EXISTS trg_sync_user_base_role_ins ON auth.users"#.to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP TRIGGER IF EXISTS trg_sync_user_base_role_upd ON auth.users"#.to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"CREATE TRIGGER trg_sync_user_base_role_ins
            AFTER INSERT ON auth.users
            FOR EACH ROW EXECUTE FUNCTION auth.sync_user_base_role()"#.to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"CREATE TRIGGER trg_sync_user_base_role_upd
            AFTER UPDATE OF role ON auth.users
            FOR EACH ROW EXECUTE FUNCTION auth.sync_user_base_role()"#.to_string(),
        ))
        .await?;

        // 6) Create function to enforce that auth.user_roles always equals base role
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            CREATE OR REPLACE FUNCTION auth.enforce_base_only_user_roles()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $$
            DECLARE
              base_code text;
              new_code  text;
            BEGIN
              SELECT u.role::text INTO base_code FROM auth.users u WHERE u.id = NEW.user_id;
              SELECT r.code      INTO new_code  FROM auth.roles r WHERE r.id = NEW.role_id;

              IF base_code IS NULL OR new_code IS NULL THEN
                RAISE EXCEPTION 'Invalid user or role reference for user_roles';
              END IF;

              IF new_code <> base_code THEN
                RAISE EXCEPTION 'auth.user_roles may only contain the base role (%). Got % for user %',
                                base_code, new_code, NEW.user_id;
              END IF;

              RETURN NEW;
            END;
            $$;
            "#.to_string(),
        ))
        .await?;

        // 7) Enforce via BEFORE triggers on auth.user_roles (split into single statements)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP TRIGGER IF EXISTS trg_enforce_base_only_user_roles_ins ON auth.user_roles"#.to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP TRIGGER IF EXISTS trg_enforce_base_only_user_roles_upd ON auth.user_roles"#.to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"CREATE TRIGGER trg_enforce_base_only_user_roles_ins
            BEFORE INSERT ON auth.user_roles
            FOR EACH ROW EXECUTE FUNCTION auth.enforce_base_only_user_roles()"#.to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"CREATE TRIGGER trg_enforce_base_only_user_roles_upd
            BEFORE UPDATE ON auth.user_roles
            FOR EACH ROW EXECUTE FUNCTION auth.enforce_base_only_user_roles()"#.to_string(),
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

        // Drop triggers on user_roles (split into single statements)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP TRIGGER IF EXISTS trg_enforce_base_only_user_roles_ins ON auth.user_roles"#.to_string(),
        ))
        .await?;
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP TRIGGER IF EXISTS trg_enforce_base_only_user_roles_upd ON auth.user_roles"#.to_string(),
        ))
        .await?;

        // Drop enforcement function
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP FUNCTION IF EXISTS auth.enforce_base_only_user_roles();"#.to_string(),
        ))
        .await?;

        // Drop triggers on users (split into single statements)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP TRIGGER IF EXISTS trg_sync_user_base_role_ins ON auth.users"#.to_string(),
        ))
        .await?;
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP TRIGGER IF EXISTS trg_sync_user_base_role_upd ON auth.users"#.to_string(),
        ))
        .await?;

        // Drop sync function
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP FUNCTION IF EXISTS auth.sync_user_base_role();"#.to_string(),
        ))
        .await?;

        // Drop unique index
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP INDEX IF EXISTS auth.ux_user_roles_user_id;"#.to_string(),
        ))
        .await?;

        Ok(())
    }
}

