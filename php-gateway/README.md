# 🐘 Archivio Parlante — PHP Gateway

Thin API Gateway layer per authentication, authorization, session management, rate limiting, e proxy verso Rust Engine.

## Features

### Fase 3.1 - Gateway Scaffolding
- ✅ **Slim 4** Framework (PSR-7/PSR-15)
- ✅ **PHP-DI** Dependency Injection container
- ✅ **Monolog** Structured logging
- ✅ **Guzzle** HTTP client per proxy
- ✅ **PHPUnit** Testing framework (coverage target 80%)
- ✅ **PHPStan Level 8** Static analysis
- ✅ **PSR-12** Coding standard

### Fase 3.2 - JWT Authentication ✅
- ✅ **JWT** access tokens (15 min) + refresh tokens (7 days)
- ✅ **Bcrypt** password hashing (cost factor 12)
- ✅ **Redis** session management with TTL
- ✅ **Rate limiting** (5 attempts per 15 min per IP)
- ✅ **OWASP ASVS L2** compliant
- ✅ **firebase/php-jwt** v7 (MIT license)
- ✅ **Predis** v2 for Redis operations

## Quick Start

### Development

```bash
cd php-gateway

# Install dependencies
composer install

# Copy .env
cp .env.example .env

# Run tests
composer test

# PHPStan
composer phpstan

# Code style check
composer cs-check
```

### Docker

```bash
# Build
docker build -t archivio-php-gateway .

# Run
docker run -p 8080:80 \
  -e RUST_ENGINE_URL=http://rust-engine:8090 \
  archivio-php-gateway
```

## Authentication Flow

```
┌─────────┐                    ┌────────────┐                    ┌───────┐
│ Client  │                    │ PHP Gateway│                    │ Redis │
└────┬────┘                    └─────┬──────┘                    └───┬───┘
     │                               │                               │
     │  POST /api/auth/register      │                               │
     ├──────────────────────────────>│                               │
     │  {email, password, name}      │                               │
     │                               │  password_hash (bcrypt)       │
     │                               │  INSERT INTO ap_users         │
     │                               │                               │
     │                               │  generateAccessToken()        │
     │                               │  generateRefreshToken()       │
     │                               │                               │
     │                               │  SETEX refresh:{hash} userId  │
     │                               ├──────────────────────────────>│
     │                               │                               │
     │  201 {access_token,           │                               │
     │       refresh_token, user}    │                               │
     │<──────────────────────────────┤                               │
     │                               │                               │
     │  GET /api/auth/me             │                               │
     │  Authorization: Bearer <JWT>  │                               │
     ├──────────────────────────────>│                               │
     │                               │  validateAccessToken()        │
     │                               │  (verify signature + exp)     │
     │                               │                               │
     │  200 {id, email, role, ...}   │                               │
     │<──────────────────────────────┤                               │
     │                               │                               │
     │  POST /api/auth/refresh       │                               │
     │  {refresh_token}              │                               │
     ├──────────────────────────────>│                               │
     │                               │  GET refresh:{hash}           │
     │                               ├──────────────────────────────>│
     │                               │<──────────────────────────────┤
     │                               │  userId                       │
     │                               │                               │
     │                               │  generateAccessToken()        │
     │                               │                               │
     │  200 {access_token}           │                               │
     │<──────────────────────────────┤                               │
```

## API Endpoints

### Authentication (Fase 3.2)

#### Register New User
```bash
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123",  # Min 8 chars, uppercase, lowercase, digit
  "name": "Full Name"
}

Response 201:
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "64-byte hex string (128 chars)",
  "user": {
    "id": 123,
    "email": "user@example.com",
    "full_name": "Full Name",
    "role": "user"
  }
}
```

#### Login
```bash
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123"
}

Response 200:
{
  "access_token": "...",
  "refresh_token": "...",
  "user": {...}
}

Response 429 (rate limit exceeded):
{
  "error": "Too many requests",
  "message": "Maximum 5 login attempts exceeded. Please try again in 15 minutes.",
  "retry_after": 900
}
```

#### Refresh Access Token
```bash
POST /api/auth/refresh
Content-Type: application/json

{
  "refresh_token": "..."
}

Response 200:
{
  "access_token": "..." # New 15-min token
}

Response 401:
{
  "error": "Invalid or revoked refresh token"
}
```

#### Get Current User
```bash
GET /api/auth/me
Authorization: Bearer <access_token>

Response 200:
{
  "id": 123,
  "email": "user@example.com",
  "full_name": "Full Name",
  "role": "user",
  "is_active": true,
  "last_login_at": "2026-04-22 10:30:00",
  "created_at": "2026-01-15 08:00:00"
}
```

#### Logout
```bash
POST /api/auth/logout
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "refresh_token": "..."
}

Response 204 (No Content)
```

### Health Check (Fase 3.1)

### `GET /health`

Health check endpoint.

**Response:**
```json
{
  "status": "ok",
  "service": "php-gateway",
  "version": "0.1.0",
  "timestamp": 1735689600,
  "rust_engine": "connected"
}
```

## Architecture

```
php-gateway/
├── public/
│   ├── index.php          # Entry point
│   └── .htaccess          # Apache rewrite rules
├── src/
│   ├── Controller/
│   │   └── HealthController.php
│   ├── Service/
│   │   └── RustEngineProxy.php
│   └── Middleware/        # TODO: Fase 3.2
├── config/
│   ├── container.php      # DI container
│   ├── routes.php         # Route definitions
│   └── middleware.php     # Middleware stack
├── tests/
│   └── Unit/
│       ├── HealthControllerTest.php
│       └── RustEngineProxyTest.php
├── composer.json
├── phpunit.xml
└── Dockerfile
```

## Configuration

Environment variables (`.env`):

```env
APP_ENV=dev
APP_DEBUG=true

JWT_SECRET=your-secret-key
RUST_ENGINE_URL=http://rust-engine:8090
RUST_ENGINE_INTERNAL_TOKEN=your-token

MYSQL_HOST=mysql
MYSQL_DB=archivio_parlante_x
MYSQL_USER=root
MYSQL_PASSWORD=

REDIS_URL=redis://redis:6379

RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_SECONDS=60
```

## Testing

```bash
# Run all tests
composer test

# With coverage
vendor/bin/phpunit --coverage-html build/coverage

# Specific test
vendor/bin/phpunit tests/Unit/HealthControllerTest.php
```

**Coverage target**: 80%

## Code Quality

```bash
# PHPStan (Level 8)
composer phpstan

# Code style check (PSR-12)
composer cs-check

# Auto-fix code style
composer cs-fix
```

## Future Phases

- **Fase 3.2**: JWT Authentication & Session management
- **Fase 3.3**: Rate limiting middleware
- **Fase 3.4**: Proxy routes to Rust engine (/query, /ingest, /compare)
- **Fase 3.5**: Audit logging to MySQL

## License

MIT — See [LICENSE](../LICENSE)
