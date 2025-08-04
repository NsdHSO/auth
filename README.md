# Auth Service

A Rust-based authentication service built with Actix Web and SeaORM.

## Features

- User registration and authentication
- JWT token management
- Session management
- Password hashing and validation
- Email validation
- Database migrations
- Docker support
- CORS configuration

## Project Structure

```
auth/
├── migration/          # Database migrations
├── src/
│   ├── db/            # Database configuration
│   ├── error_handler.rs # Custom error handling
│   ├── http_response/ # HTTP response utilities
│   ├── utils/         # Utility functions
│   └── main.rs        # Application entry point
├── Dockerfile         # Docker configuration
├── docker-compose.test.yml # Test environment
└── README.md
```

## Components

- **Utils**: Helper functions for authentication, validation, and date parsing
- **HTTP Response**: Standardized HTTP response handling
- **DB**: Database connection and configuration
- **Error Handler**: Custom error types and response handling

## Getting Started

1. Copy the environment configuration:
   ```bash
   cp .env .env
   ```

2. Update the `.env` file with your database credentials

3. Run database migrations:
   ```bash
   cd migration
   cargo run
   ```

4. Start the application:
   ```bash
   cargo run
   ```

## Database Migration

The project includes SeaORM migrations for setting up the authentication database schema:

- Users table with roles and status
- Tokens table for authentication tokens
- Sessions table for session management

## Docker

Build and run with Docker:

```bash
docker build -t auth .
docker run -p 5000:5000 auth
```

For testing:
```bash
docker-compose -f docker-compose.test.yml up
```

## API Endpoints

- `GET /v1/health` - Health check endpoint

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `HOST` - Server host (default: 127.0.0.1)
- `PORT` - Server port (default: 5000)
- `RUST_LOG` - Log level (default: debug)
