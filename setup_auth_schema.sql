-- Create auth schema if it doesn't exist
CREATE SCHEMA IF NOT EXISTS auth;

-- This ensures SeaORM will use auth.seaql_migrations instead of public.seaql_migrations
SET search_path TO auth, public;
