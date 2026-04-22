# 🐘 Archivio Parlante — PHP Gateway

Thin API Gateway layer per authentication, authorization, session management, rate limiting, e proxy verso Rust Engine.

## Features (Fase 3.1)

- ✅ **Slim 4** Framework (PSR-7/PSR-15)
- ✅ **PHP-DI** Dependency Injection container
- ✅ **Monolog** Structured logging
- ✅ **Guzzle** HTTP client per proxy
- ✅ **PHPUnit** Testing framework (coverage target 80%)
- ✅ **PHPStan Level 8** Static analysis
- ✅ **PSR-12** Coding standard

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

## API Endpoints

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
