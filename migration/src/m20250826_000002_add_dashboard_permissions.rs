use sea_orm_migration::prelude::*;
use ::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Ensure we are operating on auth schema
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "SET search_path TO auth, public;".to_string(),
        ))
        .await?;

        // Insert appointment permissions
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO auth.permissions (code, description)
            VALUES
              ('dashboard.create', 'Create dashboard'),
              ('dashboard.read', 'Read dashboard'),
              ('dashboard.update', 'Update dashboard')
            ON CONFLICT (code) DO NOTHING;
            "#.to_string(),
        ))
        .await?;

        // Map ADMIN to these new permissions
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO auth.role_permissions (role_id, permission_id)
            SELECT r.id, p.id
            FROM auth.roles r
            JOIN auth.permissions p ON p.code IN (
                'dashboard.create', 'dashboard.read', 'dashboard.update'
            )
            WHERE r.code = 'ADMIN'
            ON CONFLICT DO NOTHING;
            "#.to_string(),
        ))
        .await?;

        // Map MODERATOR to these new permissions
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO auth.role_permissions (role_id, permission_id)
            SELECT r.id, p.id
            FROM auth.roles r
            JOIN auth.permissions p ON p.code IN (
                'dashboard.create', 'dashboard.read'
            )
            WHERE r.code = 'MODERATOR'
            ON CONFLICT DO NOTHING;
            "#.to_string(),
        ))
            .await?;
        // Map MODERATOR to these new permissions
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO auth.role_permissions (role_id, permission_id)
            SELECT r.id, p.id
            FROM auth.roles r
            JOIN auth.permissions p ON p.code IN (
                'dashboard.create', 'dashboard.read'
            )
            WHERE r.code = 'MODERATOR'
            ON CONFLICT DO NOTHING;
            "#.to_string(),
        ))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Remove role-permission mappings for these codes
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DELETE FROM auth.role_permissions rp
            USING auth.roles r, auth.permissions p
            WHERE rp.role_id = r.id
              AND rp.permission_id = p.id
              AND r.code = 'ADMIN'
              AND p.code IN ('emergency.create', 'emergency.read', 'emergency.update');
            "#.to_string(),
        ))
        .await?;

        // Remove the permissions themselves
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DELETE FROM auth.permissions p
            WHERE p.code IN ('emergency.create', 'emergency.read', 'emergency.update');
            "#.to_string(),
        ))
        .await?;

        Ok(())
    }
}
