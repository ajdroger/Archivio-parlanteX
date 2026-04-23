<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Integration;

use PHPUnit\Framework\TestCase;

/**
 * Integration test for complete authentication flow
 *
 * Tests: register → login → access /me → refresh → logout
 *
 * Note: Requires MySQL and Redis to be running.
 * Run with: composer test --testsuite integration
 */
final class AuthFlowTest extends TestCase
{
    public function testFullAuthenticationFlow(): void
    {
        // This is a placeholder for integration tests
        // Full implementation requires real database and Redis connection
        // Mark as incomplete for now
        $this->markTestIncomplete('Integration test requires MySQL and Redis services running');

        // Example flow (to be implemented when services are available):
        //
        // 1. Register new user
        // POST /api/auth/register
        // Expect: 201 with access_token, refresh_token, user data
        //
        // 2. Login with credentials
        // POST /api/auth/login
        // Expect: 200 with tokens
        //
        // 3. Access protected endpoint
        // GET /api/auth/me with Authorization header
        // Expect: 200 with user data
        //
        // 4. Refresh access token
        // POST /api/auth/refresh with refresh_token
        // Expect: 200 with new access_token
        //
        // 5. Logout
        // POST /api/auth/logout with refresh_token
        // Expect: 204
        //
        // 6. Verify logout
        // POST /api/auth/refresh with revoked refresh_token
        // Expect: 401
    }
}
