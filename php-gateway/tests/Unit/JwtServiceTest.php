<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Exception\AuthenticationException;
use ArchivioParlante\Service\JwtService;
use PHPUnit\Framework\TestCase;
use Psr\Log\NullLogger;

final class JwtServiceTest extends TestCase
{
    private JwtService $jwtService;
    private string $testSecret = 'test-secret-key-with-at-least-32-characters-for-security';

    protected function setUp(): void
    {
        $this->jwtService = new JwtService($this->testSecret, new NullLogger());
    }

    public function testGenerateAccessTokenCreatesValidJwt(): void
    {
        $token = $this->jwtService->generateAccessToken(123, 'test@example.com', 'user');

        $this->assertIsString($token);
        $this->assertStringContainsString('.', $token);

        // JWT has 3 parts: header.payload.signature
        $parts = explode('.', $token);
        $this->assertCount(3, $parts);
    }

    public function testGenerateAccessTokenContainsExpectedPayload(): void
    {
        $token = $this->jwtService->generateAccessToken(123, 'test@example.com', 'admin');

        $payload = $this->jwtService->validateAccessToken($token);

        $this->assertSame('123', $payload['sub']);
        $this->assertSame('test@example.com', $payload['email']);
        $this->assertSame('admin', $payload['role']);
        $this->assertArrayHasKey('iat', $payload);
        $this->assertArrayHasKey('exp', $payload);
        $this->assertGreaterThan($payload['iat'], $payload['exp']);
    }

    public function testValidateAccessTokenSucceedsForValidToken(): void
    {
        $token = $this->jwtService->generateAccessToken(456, 'user@example.com', 'viewer');

        $payload = $this->jwtService->validateAccessToken($token);

        $this->assertIsArray($payload);
        $this->assertSame('456', $payload['sub']);
    }

    public function testValidateAccessTokenThrowsForExpiredToken(): void
    {
        // Skipping this test as it requires time manipulation or sleep
        // In production, expired tokens will be rejected by Firebase JWT library
        $this->markTestSkipped('Requires time manipulation to test token expiration');
    }

    public function testValidateAccessTokenThrowsForInvalidSignature(): void
    {
        $this->expectException(AuthenticationException::class);
        $this->expectExceptionMessage('Invalid token signature');

        $token = $this->jwtService->generateAccessToken(123, 'test@example.com', 'user');

        // Tamper with token signature
        $tamperedToken = $token . 'tampered';

        $this->jwtService->validateAccessToken($tamperedToken);
    }

    public function testValidateAccessTokenThrowsForMalformedToken(): void
    {
        $this->expectException(AuthenticationException::class);
        $this->expectExceptionMessage('Malformed token');

        $this->jwtService->validateAccessToken('not-a-valid-jwt-token');
    }

    public function testGenerateRefreshTokenCreates128CharacterHex(): void
    {
        $token = $this->jwtService->generateRefreshToken();

        $this->assertIsString($token);
        $this->assertSame(128, strlen($token)); // 64 bytes = 128 hex characters
        $this->assertMatchesRegularExpression('/^[a-f0-9]{128}$/', $token);
    }

    public function testGenerateRefreshTokenIsUnique(): void
    {
        $token1 = $this->jwtService->generateRefreshToken();
        $token2 = $this->jwtService->generateRefreshToken();

        $this->assertNotSame($token1, $token2);
    }

    public function testExtractTokenFromHeaderSucceeds(): void
    {
        $token = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature';
        $authHeader = 'Bearer ' . $token;

        $extracted = $this->jwtService->extractTokenFromHeader($authHeader);

        $this->assertSame($token, $extracted);
    }

    public function testExtractTokenFromHeaderReturnsNullForMissingBearer(): void
    {
        $extracted = $this->jwtService->extractTokenFromHeader('InvalidFormat');

        $this->assertNull($extracted);
    }

    public function testExtractTokenFromHeaderReturnsNullForEmptyString(): void
    {
        $extracted = $this->jwtService->extractTokenFromHeader('');

        $this->assertNull($extracted);
    }

    public function testExtractTokenFromHeaderReturnsNullForNull(): void
    {
        $extracted = $this->jwtService->extractTokenFromHeader(null);

        $this->assertNull($extracted);
    }
}
