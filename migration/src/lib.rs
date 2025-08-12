pub use sea_orm_migration::prelude::*;

mod m20250804_000001_create_auth_tables;
mod m20250807_000001_add_login_history;
mod m20250811_000001_add_refresh_token_to_tokens;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250804_000001_create_auth_tables::Migration),
            Box::new(m20250807_000001_add_login_history::Migration),
            Box::new(m20250811_000001_add_refresh_token_to_tokens::Migration),
        ]
    }
}
