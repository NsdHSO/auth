use sea_orm_migration::prelude::*;
use ::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Ensure we operate in the auth schema
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "SET search_path TO auth, public;".to_string(),
        ))
        .await?;

        // 1) Drop enforcement triggers so multiple roles are allowed
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

        // 2) Drop the enforcement function
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP FUNCTION IF EXISTS auth.enforce_base_only_user_roles();"#.to_string(),
        ))
        .await?;

        // 3) Replace unique(user_id) with unique(user_id, role_id)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP INDEX IF EXISTS auth.ux_user_roles_user_id;"#.to_string(),
        ))
        .await?;
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM pg_indexes
                    WHERE schemaname = 'auth' AND indexname = 'ux_user_roles_user_role_id'
                ) THEN
                    CREATE UNIQUE INDEX ux_user_roles_user_role_id ON auth.user_roles (user_id, role_id);
                END IF;
            END $$;
            "#.to_string(),
        ))
        .await?;

        // 4) Relax sync function: only ensure base role exists, do not delete others
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            CREATE OR REPLACE FUNCTION auth.sync_user_base_role()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $$
            BEGIN
              IF TG_OP = 'INSERT' THEN
                INSERT INTO auth.user_roles (user_id, role_id)
                SELECT NEW.id, r.id
                FROM auth.roles r
                WHERE r.code = NEW.role::text
                ON CONFLICT (user_id, role_id) DO NOTHING;
                RETURN NEW;
              END IF;

              IF TG_OP = 'UPDATE' AND NEW.role <> OLD.role THEN
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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "SET search_path TO auth, public;".to_string(),
        ))
        .await?;

        // Recreate unique(user_id) and drop composite index
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DROP INDEX IF EXISTS auth.ux_user_roles_user_role_id;"#.to_string(),
        ))
        .await?;
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

        // Restore strict sync function: keep only base role
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            CREATE OR REPLACE FUNCTION auth.sync_user_base_role()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $$
            BEGIN
              IF TG_OP = 'INSERT' THEN
                DELETE FROM auth.user_roles WHERE user_id = NEW.id;
                INSERT INTO auth.user_roles (user_id, role_id)
                SELECT NEW.id, r.id
                FROM auth.roles r
                WHERE r.code = NEW.role::text
                ON CONFLICT (user_id, role_id) DO NOTHING;
                RETURN NEW;
              END IF;

              IF TG_OP = 'UPDATE' AND NEW.role <> OLD.role THEN
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

        // Recreate enforcement function and triggers to restrict to base role only
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
}

