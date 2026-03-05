#!/usr/bin/env bash
set -Eeuo pipefail

# Load DATABASE_URL from environment or .env
if [[ -z "${DATABASE_URL:-}" ]]; then
  if [[ -f ".env" ]]; then
    set -a
    source .env
    set +a
  fi
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "ERROR: DATABASE_URL is not set" >&2
  exit 1
fi

echo "Creating auth schema if it doesn't exist..."

# Create a temporary Rust program to create the schema
cat > /tmp/create_auth_schema.rs << 'RUSTEOF'
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = env::var("DATABASE_URL")?;

    let pool = sqlx::PgPool::connect(&db_url).await?;

    sqlx::query("CREATE SCHEMA IF NOT EXISTS auth")
        .execute(&pool)
        .await?;

    println!("✓ Auth schema created successfully");

    Ok(())
}
RUSTEOF

# Create Cargo.toml for the helper
cat > /tmp/Cargo.toml << 'CARGOEOF'
[package]
name = "create-schema"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls"] }
tokio = { version = "1", features = ["full"] }
CARGOEOF

# Run the helper
cd /tmp
DATABASE_URL="$DATABASE_URL" cargo run --quiet --manifest-path /tmp/Cargo.toml 2>/dev/null

echo "Schema setup complete!"
