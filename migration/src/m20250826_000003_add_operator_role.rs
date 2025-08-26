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

        // Add OPERATOR to the Postgres enum for users.role
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "ALTER TYPE auth.user_role ADD VALUE IF NOT EXISTS 'OPERATOR';".to_string(),
        ))
        .await?;

        // Insert OPERATOR role
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO auth.roles (code, description, priority)
            VALUES ('OPERATOR', 'Appointment and Dashboard access', 30)
            ON CONFLICT (code) DO NOTHING;
            "#.to_string(),
        ))
        .await?;

        // Map OPERATOR -> appointment.* and dashboard.* permissions
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO auth.role_permissions (role_id, permission_id)
            SELECT r.id, p.id
            FROM auth.roles r
            JOIN auth.permissions p ON p.code IN (
                'appointment.create', 'appointment.read', 'appointment.update',
                'dashboard.create', 'dashboard.read', 'dashboard.update'
            )
            WHERE r.code = 'OPERATOR'
            ON CONFLICT DO NOTHING;
            "#.to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Remove role-permission mappings for OPERATOR
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DELETE FROM auth.role_permissions rp
            USING auth.roles r
            WHERE rp.role_id = r.id
              AND r.code = 'OPERATOR';
            "#.to_string(),
        ))
        .await?;

        // Remove OPERATOR role itself (will cascade to user_roles if any)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DELETE FROM auth.roles WHERE code = 'OPERATOR';".to_string(),
        ))
        .await?;

        // Note: We do not remove the enum value from Postgres type in down() for safety.
        Ok(())
    }
}

