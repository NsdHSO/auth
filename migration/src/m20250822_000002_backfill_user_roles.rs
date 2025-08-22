use sea_orm_migration::prelude::*;
use ::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Ensure auth schema is in search path
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "SET search_path TO auth, public;".to_string(),
        ))
        .await?;

        // Backfill user_roles for users with role = 'USER'
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO auth.user_roles (user_id, role_id)
            SELECT u.id, r.id
            FROM auth.users u
            JOIN auth.roles r ON r.code = 'USER'
            LEFT JOIN auth.user_roles ur ON ur.user_id = u.id AND ur.role_id = r.id
            WHERE u.role = 'USER' AND ur.user_id IS NULL;
            "#.to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            DELETE FROM auth.user_roles ur
            USING auth.roles r
            WHERE ur.role_id = r.id AND r.code = 'USER';
            "#.to_string(),
        ))
        .await?;

        Ok(())
    }
}
