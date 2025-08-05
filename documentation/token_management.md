# Token Management

## Overview
Tokens are a crucial part of the authentication and authorization process. This document provides a detailed explanation of the types of tokens used, their purpose, structure, and management practices.

## Types of Tokens

### 1. Access Token
- **Purpose**: Used to access protected resources.
- **Type**: JWT (JSON Web Token).
- **Expiration**: Typically short-lived (e.g., 15 minutes).
- **Structure**: Three parts - Header, Payload, and Signature.

### 2. Refresh Token
- **Purpose**: To obtain a new access token without re-authenticating.
- **Type**: Secure random string (not a JWT).
- **Expiration**: Long-lived compared to access tokens (e.g., 7 days).

## Access Token: JWT Structure

### 1. Header
- **Algorithm**: Specifies the signing algorithm, e.g., HS256.
- **Type**: Always set to JWT.

```json
{
  "alg": "HS256",
  "typ": "JWT"
}
```

### 2. Payload
- Contains claims about the user and token metadata.
- **Registered Claims**:
  - `iss`: Issuer of the token.
  - `sub`: Subject (usually user ID).
  - `aud`: Audience for which the token is intended.
  - `exp`: Expiration time (epoch).
  - `nbf`: Not before time (epoch).
  - `iat`: Issued at time (epoch).
  - `jti`: JWT ID (unique identifier for the token).

**Example**:
```json
{
  "sub": "user_id",
  "name": "John Doe",
  "admin": true,
  "exp": 1615149163
}
```

### 3. Signature
- Created by hashing the encoded header and payload with a secret.
- **Formula**:
  - HMACSHA256(
    base64UrlEncode(header) + "." +
    base64UrlEncode(payload),
    secret
  )

## Refresh Token

- **Storage**: Stored securely, often as a hashed value.
- **Usage**: Sent to a dedicated endpoint for acquiring a new access token.

## Best Practices

### Security
- **Confidentiality**: Tokens must be kept secure.
- **Renewal**: Regularly update tokens to reduce risk.
- **Scopes**: Limit token permissions to only necessary actions.
- **Revoke Tokens**: Provide a mechanism to revoke tokens if needed.

### Storage
- **Access Tokens**: Store in memory (never persist).
- **Refresh Tokens**: Store securely (e.g., httpOnly cookies).

## Rate Limiting and Token Rotation
- **Throttle** API calls to avoid abuse.
- **Token Rotation**: Implement rotation to regularly issue new refresh tokens.

## Revocation Mechanism
- Maintain a blacklist of revoked tokens.
- Check the blacklist before allowing actions with a token.

## Environment Variables for Token Configuration

```env
# JWT Settings
JWT_SECRET_KEY=your-secure-jwt-secret
JWT_ALGORITHM=HS256
JWT_ACCESS_TOKEN_EXPIRE_MINUTES=15

# Refresh Token Settings
REFRESH_TOKEN_EXPIRE_DAYS=7
SECURE_REFRESHTOKEN_STORAGE_PATH=/path/to/secure/location
```

For more information on JWTs, see [JWT.io](https://jwt.io/).

Use these guidelines to manage tokens securely and efficiently within your application.
