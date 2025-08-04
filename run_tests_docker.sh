#!/bin/bash

# Script to run tests using Docker Compose
# This ensures a clean test environment with a separate database

set -e

echo "🧪 Starting auth service tests with Docker..."

# Build and run tests
docker-compose -f docker-compose.test.yml down --volumes
docker-compose -f docker-compose.test.yml up --build --abort-on-container-exit

# Clean up
docker-compose -f docker-compose.test.yml down --volumes

echo "✅ Auth tests completed!"
