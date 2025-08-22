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
├── migration/                # Database migrations
├── src/
│   ├── components/
│   │   ├── auth/            # Auth routes and service
│   │   ├── config/          # ConfigService (env loader)
│   │   ├── mail_send/       # Email sending (lettre)
│   │   └── tokens/          # Token management
│   ├── db/                  # Database configuration
│   ├── entity/              # SeaORM entities
│   ├── http_response/       # HTTP response utilities and error handler
│   ├── utils/               # Utility functions
│   └── main.rs              # Application entry point
├── Dockerfile
├── docker-compose.test.yml
└── README.md
```

## Components

- Auth: Registration, login, logout, refresh, email verification
- Tokens: Creation and validation of verification/access/refresh tokens
- Config: ConfigService for centralized environment/config loading used across the app
- Mail Send: Outbound email via lettre (SMTP). Verification link sent as {PORT_HOST}/v1/auth/verify/{token}
- HTTP Response: Standardized HTTP response handling and error mapping
- DB: Database connection and configuration
- Utils: Helper functions for authentication, validation, and dates
- Entities: SeaORM entities and enums

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
docker run -p 4100:4100 auth
```

For testing:
```bash
docker-compose -f docker-compose.test.yml up
```

## API Architecture

### Core Authentication Endpoints (8 endpoints)

#### **Authentication Flow**
```
POST /v1/auth/register          # User registration
POST /v1/auth/login             # User login
POST /v1/auth/logout            # User logout
POST /v1/auth/refresh           # Refresh access token
```

#### **Email Verification**
```
POST /v1/auth/verify-email      # Send verification email
GET  /v1/auth/verify/{token}    # Verify email with token
```

#### **Password Management**
```
POST /v1/auth/forgot-password   # Request password reset
POST /v1/auth/reset-password    # Reset password with token
```

### User Profile Management (4 endpoints)

#### **Profile Operations**
```
GET    /v1/users/profile        # Get current user profile
PUT    /v1/users/profile        # Update user profile
PATCH  /v1/users/password       # Change password
DELETE /v1/users/account        # Delete user account
```

### Session & Token Management (5 endpoints)

#### **Session Management**
```
GET    /v1/sessions             # List user sessions
DELETE /v1/sessions/{id}        # Revoke specific session
DELETE /v1/sessions/all         # Revoke all sessions
```

#### **Token Management**
```
GET    /v1/tokens               # List user tokens
DELETE /v1/tokens/{id}          # Revoke specific token
```

### Admin & User Management (7 endpoints)

#### **User Administration** (Admin only)
```
GET    /v1/admin/users          # List all users (paginated)
GET    /v1/admin/users/{id}     # Get specific user
PATCH  /v1/admin/users/{id}/status    # Update user status
PATCH  /v1/admin/users/{id}/role      # Update user role
DELETE /v1/admin/users/{id}     # Delete user
GET    /v1/admin/sessions       # List all sessions
GET    /v1/admin/tokens         # List all tokens
```

### System & Health (2 endpoints)

#### **System Monitoring**
```
GET /v1/health                  # Health check
GET /v1/health/detailed         # Detailed health status
```

---

## Complete API Reference

### **Authentication Endpoints**

#### `POST /v1/auth/register`
**Purpose**: Register a new user account

**Request Body**:
```json
{
  "email": "user@example.com",
  "username": "johndoe",
  "password": "SecurePass123!",
  "first_name": "John",
  "last_name": "Doe"
}
```

**Response**: `201 Created`
```json
{
  "message": {
    "user_id": "uuid",
    "email": "user@example.com",
    "username": "johndoe",
    "status": "PENDING_VERIFICATION"
  },
  "code": 201
}
```

#### `POST /v1/auth/login`
**Purpose**: Authenticate user and get tokens

**Request Body**:
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

**Response**: `200 OK`
```json
{
  "message": {
    "access_token": "jwt_access_token",
    "refresh_token": "jwt_refresh_token",
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "username": "johndoe",
      "role": "USER",
      "status": "ACTIVE"
    },
    "expires_in": 900
  },
  "code": 200
}
```

#### `POST /v1/auth/logout`
**Purpose**: Logout user and invalidate tokens

**Headers**: `Authorization: Bearer {access_token}`

**Response**: `200 OK`
```json
{
  "message": "Successfully logged out",
  "code": 200
}
```

#### `POST /v1/auth/refresh`
**Purpose**: Get new access token using refresh token

**Request Body**:
```json
{
  "refresh_token": "jwt_refresh_token"
}
```

**Response**: `200 OK`
```json
{
  "message": {
    "access_token": "new_jwt_access_token",
    "expires_in": 900
  },
  "code": 200
}
```

---

### **Profile Management Endpoints**

#### `GET /v1/users/profile`
**Purpose**: Get current user profile

**Headers**: `Authorization: Bearer {access_token}`

**Response**: `200 OK`
```json
{
  "message": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "johndoe",
    "first_name": "John",
    "last_name": "Doe",
    "role": "USER",
    "status": "ACTIVE",
    "email_verified": true,
    "created_at": "2025-01-01T00:00:00Z",
    "last_login": "2025-01-01T12:00:00Z"
  },
  "code": 200
}
```

#### `PUT /v1/users/profile`
**Purpose**: Update user profile

**Headers**: `Authorization: Bearer {access_token}`

**Request Body**:
```json
{
  "first_name": "John",
  "last_name": "Smith",
  "username": "johnsmith"
}
```

**Response**: `200 OK`

---

### **Session Management Endpoints**

#### `GET /v1/sessions`
**Purpose**: List user's active sessions

**Headers**: `Authorization: Bearer {access_token}`

**Response**: `200 OK`
```json
{
  "message": {
    "sessions": [
      {
        "id": "uuid",
        "ip_address": "192.168.1.1",
        "user_agent": "Mozilla/5.0...",
        "created_at": "2025-01-01T10:00:00Z",
        "expires_at": "2025-01-08T10:00:00Z",
        "is_current": true
      }
    ],
    "total": 1
  },
  "code": 200
}
```

---

### **Admin Endpoints**

#### `GET /v1/admin/users`
**Purpose**: List all users (paginated)

**Headers**: `Authorization: Bearer {admin_access_token}`

**Query Parameters**:
- `page` (optional): Page number (default: 1)
- `limit` (optional): Items per page (default: 20, max: 100)
- `status` (optional): Filter by status
- `role` (optional): Filter by role
- `search` (optional): Search by email/username

**Response**: `200 OK`
```json
{
  "message": {
    "users": [
      {
        "id": "uuid",
        "email": "user@example.com",
        "username": "johndoe",
        "role": "USER",
        "status": "ACTIVE",
        "email_verified": true,
        "created_at": "2025-01-01T00:00:00Z",
        "last_login": "2025-01-01T12:00:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 150,
      "pages": 8
    }
  },
  "code": 200
}
```

---

## Total API Count: **26 Endpoints**

### **Breakdown by Category**:
- 🔐 **Authentication**: 8 endpoints
- 👤 **User Profile**: 4 endpoints  
- 🎫 **Session/Token Management**: 5 endpoints
- 👨‍💼 **Admin Operations**: 7 endpoints
- 🏥 **System Health**: 2 endpoints

---

## Authentication & Authorization

### **Security Headers**
- `Authorization: Bearer {access_token}` - Required for protected endpoints
- `Content-Type: application/json` - For request body

### **Access Levels**
- 🟢 **Public**: Registration, login, email verification, password reset
- 🟡 **User**: Profile management, session management, logout
- 🔴 **Admin**: User management, system monitoring

### **Token Types**
- **Access Token**: Short-lived (15 minutes), used for API access
- **Refresh Token**: Long-lived (7 days), used to get new access tokens
- **Verification Token**: One-time use (24 hours), for email verification
- **Reset Token**: One-time use (1 hour), for password reset

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `HOST` - Server host (default: 127.0.0.1)
- `PORT` - Server port (default: 4100)
- `RUST_LOG` - Log level (default: debug)

## RBAC: Roles & Permissions Matrix

Below is a plain Markdown matrix (no Mermaid) of roles vs permissions.

| Permission            | ADMIN | MODERATOR | USER | GUEST |
|-----------------------|:-----:|:---------:|:----:|:-----:|
| user.read             |   ✓   |     ✓     |      |       |
| user.write            |   ✓   |           |      |       |
| session.read          |   ✓   |     ✓     |      |       |
| session.terminate     |   ✓   |     ✓     |      |       |
| token.read            |   ✓   |           |  ✓   |       |
| token.revoke          |   ✓   |           |      |       |
| project.read          |   ✓   |     ✓     |  ✓   |   ✓   |
| project.write         |   ✓   |     ✓     |      |       |
| project.delete        |   ✓   |           |      |       |
| appointment.create    |   ✓   |           |      |       |
| appointment.read      |   ✓   |           |      |       |
| appointment.update    |   ✓   |           |      |       |

Legend:
- ✓ granted
- blank not granted

Notes:
- ADMIN is granted all current permissions by default and will receive newly added permissions via migrations.
- User-specific overrides exist via `auth.user_permission_overrides` where `allow=true` adds a permission and `allow=false` removes it, regardless of role.
- The JWT access token embeds the effective `perms` and `roles` claims so microservices can authorize without DB access.
