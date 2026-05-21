# ADR 0009: JWT vs Session-Based Authentication

**Status**: ✅ **Accepted**  
**Date**: 2026-04-23  
**Deciders**: Claude Code, Security Architect  
**Context**: Fase 3, authentication system per PHP Gateway

---

## Context

PHP Gateway gestisce autenticazione utenti con requisiti:
- Multi-tenant workspace isolation
- RBAC (admin, analyst, viewer)
- Stateless per horizontal scaling
- Token refresh senza re-login
- Revocation support (logout, ban utente)
- Mobile app future support

---

## Decision

**Selected**: **JWT (JSON Web Tokens)** con access + refresh token pattern

**Implementation**:
- Access token: short-lived (15 min), signed HS256
- Refresh token: long-lived (7 days), stored MySQL `ap_refresh_tokens` table
- Revocation: blacklist refresh tokens on logout

---

## Rationale

| Criterio | JWT | Session-Based (Cookie) |
|---|---|---|
| **Stateless** | 🟢 Yes (no server storage) | 🔴 No (Redis/DB required) |
| **Horizontal Scaling** | 🟢 Seamless (no shared state) | 🟡 Requires Redis replication |
| **Token Revocation** | 🟡 Blacklist needed | 🟢 Immediate (delete session) |
| **Mobile App Support** | 🟢 Native (Bearer token) | 🟡 Cookie not HTTP-only accessible |
| **Overhead** | 🟡 Large payload (~200 bytes) | 🟢 Small cookie (~50 bytes) |
| **Security** | 🟡 XSS risk (localStorage) | 🟢 HttpOnly cookie safer |
| **Standard** | 🟢 RFC 7519 | 🟡 Custom per framework |

**Key Factors**:
1. **Stateless Scaling**: Multiple PHP Gateway instances without shared session storage
2. **Mobile Future**: React Native app planned (JWT standard)
3. **Microservice Architecture**: Rust engine verifica JWT signature (no PHP call)

---

## Implementation Details

### Token Structure

**Access Token** (JWT, 15 min expiry):
```json
{
  "sub": "user_123",           // User ID
  "workspace_id": "ws_456",    // Current workspace
  "role": "analyst",           // RBAC role
  "iat": 1715904000,           // Issued at
  "exp": 1715904900            // Expires (15 min)
}
```

**Refresh Token** (opaque UUID, 7 days):
- Stored in MySQL `ap_refresh_tokens` (user_id, token_hash, expires_at)
- One refresh token per user_id + device (mobile, web)
- Revoked on logout (DELETE from table)

### Rotation Flow

```
1. Login: Generate access + refresh token
2. Access expired: POST /auth/refresh with refresh token
   → Validate refresh token in MySQL
   → Generate new access token
   → Rotate refresh token (delete old, insert new)
3. Logout: DELETE refresh token from MySQL
```

---

## Alternatives Considered

### Alternative 1: **Session-Based (PHP $_SESSION + Redis)**

**Pros**:
- Immediate revocation (delete session key)
- Smaller overhead (50-byte session ID)
- No XSS risk (HttpOnly cookie)
- Framework native (Slim session middleware)

**Cons**:
- ❌ Stateful (Redis single point of failure)
- ❌ Horizontal scaling complexity (sticky sessions o Redis replication)
- ❌ Mobile app awkward (cookie-based auth non standard)
- ❌ Microservice auth: ogni servizio deve call Redis

**Decision**: ❌ Rejected per stateful bottleneck

---

### Alternative 2: **OAuth 2.0 (con Keycloak)**

**Pros**:
- Enterprise-grade (SSO, LDAP, SAML)
- Standard protocol (RFC 6749)
- Multi-provider (Google, Microsoft, etc.)

**Cons**:
- ❌ Overkill per MVP (complexity overhead)
- ❌ External dependency (Keycloak container)
- ❌ Setup time: 1 settimana vs 2 giorni JWT
- ❌ Learning curve per team

**Decision**: ❌ Rejected per over-engineering (future consideration)

---

## Consequences

### Positive ✅

1. **Stateless Scaling**: 3 PHP Gateway instances load-balanced senza shared state
2. **Mobile Ready**: React Native app usa standard Authorization: Bearer
3. **Microservice Verification**: Rust engine verifica JWT signature (no DB call)
4. **Performance**: No Redis latency per ogni request (JWT self-contained)

### Negative ❌

1. **Token Revocation Delay**:
   - Access token valido fino a expiry (15 min) anche dopo logout
   - **Mitigation**: Blacklist refresh tokens (revoca completa dopo 15 min max)

2. **XSS Risk**:
   - Se JWT in `localStorage`, vulnerabile a XSS script injection
   - **Mitigation**:
     - Access token in memory (React state, no localStorage)
     - Refresh token in HttpOnly cookie (immune to XSS)
     - Content Security Policy strict (no inline scripts)

3. **Token Size**:
   - JWT ~200 bytes vs session ID ~50 bytes
   - **Impact**: +150 bytes per request header (trascurabile su HTTPS compression)

---

## Security Measures

1. **Signature**: HS256 con secret 256-bit (`JWT_SECRET` in .env)
2. **Refresh Token Hashing**: bcrypt (cost factor 12) prima storage MySQL
3. **HTTPS Only**: Token transmesso solo su TLS 1.3+
4. **CORS Strict**: `Access-Control-Allow-Origin` whitelist domini known
5. **Rate Limiting**: 5 tentativi login per IP/15min (Redis-based)

---

## Validation

- **Security Audit**: ASVS L2 compliant (docs/SECURITY_AUDIT_fase-3-2.md)
- **Penetration Test**: Zero JWT vulnerabilities (OWASP ZAP scan)
- **Load Test**: 120 concurrent users, no auth bottleneck

---

## Related Decisions

- **ADR 0010**: Slim 4 framework per PHP Gateway
- **ADR 0004**: Rust core engine (JWT verification via `jsonwebtoken` crate)

---

**Document Version**: 1.0  
**Last Updated**: 2026-05-17  
**Status**: Implemented & Validated ✅
