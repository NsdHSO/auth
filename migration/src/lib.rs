pub use sea_orm_migration::prelude::*;

mod m20250804_000001_create_auth_tables;
mod m20250807_000001_add_login_history;
mod m20250811_000001_add_refresh_token_to_tokens;
mod m20250822_000001_create_rbac_tables;
mod m20250822_000002_backfill_user_roles;
mod m20250822_000003_add_appointment_permissions;
mod m20250826_000001_add_emergency_permissions;
mod m20250826_000002_add_dashboard_permissions;
mod m20250826_000003_add_operator_role;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250804_000001_create_auth_tables::Migration),
            Box::new(m20250807_000001_add_login_history::Migration),
            Box::new(m20250811_000001_add_refresh_token_to_tokens::Migration),
            Box::new(m20250822_000001_create_rbac_tables::Migration),
            Box::new(m20250822_000002_backfill_user_roles::Migration),
            Box::new(m20250822_000003_add_appointment_permissions::Migration),
            Box::new(m20250826_000001_add_emergency_permissions::Migration),
Box::new(m20250826_000002_add_dashboard_permissions::Migration),
            Box::new(m20250826_000003_add_operator_role::Migration),
        ]
    }
}
