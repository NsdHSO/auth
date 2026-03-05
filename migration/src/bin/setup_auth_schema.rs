use sea_orm::{Database, Statement, ConnectionTrait};
use std::env;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let db = Database::connect(&database_url).await?;

    println!("Creating auth schema if it doesn't exist...");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE SCHEMA IF NOT EXISTS auth;".to_string(),
    )).await?;

    println!("✓ Auth schema is ready");
    println!("You can now run migrations with: cargo run --manifest-path migration/Cargo.toml -- --database-url \"$DATABASE_URL\" --database-schema auth");

    Ok(())
}
