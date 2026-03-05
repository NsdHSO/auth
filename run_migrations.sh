#!/usr/bin/env bash
set -Eeuo pipefail

# Load DATABASE_URL from environment or .env (without overwriting an existing env var)
if [[ -z "${DATABASE_URL:-}" ]]; then
  if [[ -f ".env" ]]; then
    # shellcheck disable=SC1091
    set -a
    source .env
    set +a
  fi
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "ERROR: DATABASE_URL is not set. Set it in your environment or in .env" >&2
  exit 1
fi

echo "Setting up auth schema..."
# First, create the auth schema so SeaORM can use it for migration tracking
cargo run --quiet --bin setup_auth_schema --manifest-path migration/Cargo.toml

echo ""
echo "Running database migrations in auth schema..."
# Now run migrations WITH --database-schema so SeaORM uses auth.seaql_migrations
# This keeps auth migrations separate from any public schema migrations
cargo run --bin migration --manifest-path migration/Cargo.toml -- --database-url "$DATABASE_URL" --database-schema auth

echo "Migrations completed!"
