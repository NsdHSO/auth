#!/bin/bash

# Set the database URL for the test database
export DATABASE_URL="postgresql://fat_user:fat_pass@192.168.68.50:5440/fat_db"

echo "Running database migrations..."
echo "Database URL: $DATABASE_URL"

# Run the migrations, explicitly passing the variable's value
cd migration
cargo run -- --database-url "$DATABASE_URL" --database-schema auth

echo "Migrations completed!"	