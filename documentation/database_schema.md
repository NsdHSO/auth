# Database Schema

## Overview
This document outlines the database schema required for the authentication system with proper token management.

## Tables

### 1. users
Stores user account information.

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(30) UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    email_verification_token VARCHAR(255),
    email_verification_expires_at TIMESTAMP WITH TIME ZONE,
    password_reset_token VARCHAR(255),
    password_reset_expires_at TIMESTAMP WITH TIME ZONE,
    last_login_at TIMESTAMP WITH TIME ZONE,
    failed_login_attempts INTEGER DEFAULT 0,
    locked_until TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email_verification_token ON users(email_verification_token);
CREATE INDEX idx_users_password_reset_token ON users(password_reset_token);
```

### 2. refresh_tokens
Stores refresh tokens for JWT authentication.

```sql
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    device_info JSONB,
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    revoked BOOLEAN DEFAULT FALSE,
    revoked_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
CREATE INDEX idx_refresh_tokens_revoked ON refresh_tokens(revoked);
```

### 3. user_sessions (Optional - for session tracking)
Tracks active user sessions for security monitoring.

```sql
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(255) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    device_info JSONB,
    location JSONB,
    is_active BOOLEAN DEFAULT TRUE,
    last_activity_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_session_token ON user_sessions(session_token);
CREATE INDEX idx_user_sessions_is_active ON user_sessions(is_active);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
```

### 4. audit_logs (Optional - for security auditing)
Logs authentication events for security monitoring.

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(50) NOT NULL, -- 'register', 'login', 'logout', 'password_reset', etc.
    event_data JSONB,
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_success ON audit_logs(success);
```

## Required Database Operations for Register Endpoint

### 1. User Registration Process
```sql
-- 1. Check if user already exists
SELECT id FROM users WHERE email = $1 OR username = $2;

-- 2. Insert new user (if doesn't exist)
INSERT INTO users (
    email, 
    username, 
    password_hash, 
    first_name, 
    last_name,
    email_verification_token,
    email_verification_expires_at
) VALUES (
    $1, $2, $3, $4, $5, $6, $7
) RETURNING id, email, username, first_name, last_name, email_verified, created_at, updated_at;

-- 3. Create refresh token
INSERT INTO refresh_tokens (
    user_id,
    token_hash,
    device_info,
    ip_address,
    user_agent,
    expires_at
) VALUES (
    $1, $2, $3, $4, $5, $6
) RETURNING id;

-- 4. Log registration event (optional)
INSERT INTO audit_logs (
    user_id,
    event_type,
    event_data,
    ip_address,
    user_agent,
    success
) VALUES (
    $1, 'register', $2, $3, $4, TRUE
);
```

### 2. Token Management Queries

#### Generate Access Token (JWT payload data)
```sql
SELECT 
    id,
    email,
    username,
    email_verified,
    created_at
FROM users 
WHERE id = $1;
```

#### Validate Refresh Token
```sql
SELECT 
    rt.id,
    rt.user_id,
    rt.expires_at,
    rt.revoked,
    u.email,
    u.username
FROM refresh_tokens rt
JOIN users u ON rt.user_id = u.id
WHERE rt.token_hash = $1 
    AND rt.expires_at > NOW() 
    AND rt.revoked = FALSE;
```

#### Revoke Refresh Token
```sql
UPDATE refresh_tokens 
SET revoked = TRUE, revoked_at = NOW() 
WHERE token_hash = $1;
```

#### Clean Expired Tokens (Maintenance)
```sql
DELETE FROM refresh_tokens 
WHERE expires_at < NOW() OR revoked = TRUE;
```

## Environment Variables Required

```env
# Database
DATABASE_URL=postgresql://username:password@localhost:5432/auth_db

# JWT
JWT_SECRET=your-super-secret-jwt-key
JWT_ACCESS_TOKEN_EXPIRY=900  # 15 minutes
JWT_REFRESH_TOKEN_EXPIRY=604800  # 7 days

# Password hashing
BCRYPT_ROUNDS=12

# Email verification
EMAIL_VERIFICATION_EXPIRY=86400  # 24 hours
PASSWORD_RESET_EXPIRY=3600  # 1 hour
```

## Security Considerations

1. **Password Storage**: Always hash passwords using bcrypt with cost factor 12+
2. **Token Security**: Store refresh tokens hashed in database
3. **Token Rotation**: Implement refresh token rotation for better security
4. **Rate Limiting**: Implement rate limiting on registration endpoint
5. **Email Verification**: Require email verification before full account access
6. **Audit Trail**: Log all authentication events for security monitoring
7. **Session Management**: Track active sessions for security
8. **Token Cleanup**: Regularly clean expired tokens from database

## Migration Scripts

### Initial Migration
```sql
-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to tables
CREATE TRIGGER update_users_updated_at 
    BEFORE UPDATE ON users 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_refresh_tokens_updated_at 
    BEFORE UPDATE ON refresh_tokens 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```
