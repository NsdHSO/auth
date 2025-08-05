# POST /v1/auth/register

## Description
Register a new user account and return authentication tokens.

## Endpoint
```
POST /v1/auth/register
```

## Headers
```
Content-Type: application/json
```

## Request Body
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "first_name": "John",
  "last_name": "Doe",
  "username": "johndoe"
}
```

### Request Validation
- **email**: Required, valid email format, unique
- **password**: Required, minimum 8 characters, must contain:
  - At least 1 uppercase letter
  - At least 1 lowercase letter  
  - At least 1 number
  - At least 1 special character
- **first_name**: Required, 2-50 characters
- **last_name**: Required, 2-50 characters
- **username**: Optional, 3-30 characters, alphanumeric + underscore only, unique if provided

## Success Response (201 Created)
```json
{
  "success": true,
  "message": "User registered successfully",
  "data": {
    "user": {
      "id": "uuid-v4",
      "email": "user@example.com",
      "username": "johndoe",
      "first_name": "John",
      "last_name": "Doe",
      "email_verified": false,
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    },
    "tokens": {
      "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
      "refresh_token": "refresh_token_here",
      "token_type": "Bearer",
      "expires_in": 900,
      "refresh_expires_in": 604800
    }
  }
}
```

## Error Responses

### 400 Bad Request - Validation Errors
```json
{
  "success": false,
  "error": "validation_error",
  "message": "Invalid input data",
  "details": [
    {
      "field": "email",
      "message": "Invalid email format"
    },
    {
      "field": "password",
      "message": "Password must contain at least one uppercase letter"
    }
  ]
}
```

### 409 Conflict - User Already Exists
```json
{
  "success": false,
  "error": "user_exists",
  "message": "User with this email already exists"
}
```

### 500 Internal Server Error
```json
{
  "success": false,
  "error": "internal_server_error",
  "message": "An unexpected error occurred"
}
```

## Token Information

### Access Token
- **Type**: JWT (JSON Web Token)
- **Expiration**: 15 minutes (900 seconds)
- **Usage**: Include in Authorization header for protected routes
- **Format**: `Authorization: Bearer <access_token>`

### Refresh Token
- **Type**: Secure random string
- **Expiration**: 7 days (604800 seconds)
- **Usage**: Use to obtain new access tokens
- **Storage**: Store securely (httpOnly cookie recommended)

## Security Considerations
- Passwords are hashed using bcrypt with cost factor 12
- Refresh tokens are stored hashed in database
- Rate limiting: 5 requests per minute per IP
- Email verification required for full account activation

## Example Usage

### cURL
```bash
curl -X POST http://localhost:3000/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePassword123!",
    "first_name": "John",
    "last_name": "Doe",
    "username": "johndoe"
  }'
```

### JavaScript (fetch)
```javascript
const response = await fetch('/v1/auth/register', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    email: 'user@example.com',
    password: 'SecurePassword123!',
    first_name: 'John',
    last_name: 'Doe',
    username: 'johndoe'
  })
});

const data = await response.json();
```
