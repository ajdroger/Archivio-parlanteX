<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Exception\AuthenticationException;
use ArchivioParlante\Exception\ValidationException;
use ArchivioParlante\Repository\UserRepository;
use ArchivioParlante\Service\AuditLogger;
use ArchivioParlante\Service\AuthService;
use ArchivioParlante\Service\JwtService;
use ArchivioParlante\Service\RedisSessionManager;
use PHPUnit\Framework\TestCase;
use PHPUnit\Framework\MockObject\MockObject;
use Psr\Log\NullLogger;

final class AuthServiceTest extends TestCase
{
    private AuthService $authService;
    private UserRepository & MockObject $userRepository;
    private JwtService & MockObject $jwtService;
    private RedisSessionManager & MockObject $sessionManager;
    private AuditLogger & MockObject $auditLogger;

    protected function setUp(): void
    {
        $this->userRepository = $this->createMock(UserRepository::class);
        $this->jwtService = $this->createMock(JwtService::class);
        $this->sessionManager = $this->createMock(RedisSessionManager::class);
        $this->auditLogger = $this->createMock(AuditLogger::class);

        $this->authService = new AuthService(
            $this->userRepository,
            $this->jwtService,
            $this->sessionManager,
            $this->auditLogger,
            new NullLogger()
        );
    }

    public function testRegisterSuccessfullyCreatesUser(): void
    {
        $this->userRepository->expects($this->once())
            ->method('existsByEmail')
            ->with('new@example.com')
            ->willReturn(false);

        $this->userRepository->expects($this->once())
            ->method('create')
            ->with(
                'new@example.com',
                $this->callback(fn($hash) => password_verify('SecurePass123', $hash)),
                'Test User'
            )
            ->willReturn(123);

        $this->jwtService->expects($this->once())
            ->method('generateAccessToken')
            ->with(123, 'new@example.com', 'user')
            ->willReturn('access-token');

        $this->jwtService->expects($this->once())
            ->method('generateRefreshToken')
            ->willReturn('refresh-token');

        $this->sessionManager->expects($this->once())
            ->method('storeRefreshToken')
            ->with('refresh-token', 123);

        $result = $this->authService->register('new@example.com', 'SecurePass123', 'Test User', '192.168.1.1', 'PHPUnit');

        $this->assertArrayHasKey('access_token', $result);
        $this->assertArrayHasKey('refresh_token', $result);
        $this->assertArrayHasKey('user', $result);
        $this->assertSame('access-token', $result['access_token']);
        $this->assertSame('refresh-token', $result['refresh_token']);
        $this->assertSame(123, $result['user']['id']);
    }

    public function testRegisterThrowsForInvalidEmail(): void
    {
        $this->expectException(ValidationException::class);

        $this->authService->register('not-an-email', 'SecurePass123', 'Test User', '192.168.1.1', 'PHPUnit');
    }

    public function testRegisterThrowsForWeakPassword(): void
    {
        $this->expectException(ValidationException::class);

        try {
            $this->authService->register('test@example.com', 'weak', 'Test User', '192.168.1.1', 'PHPUnit');
        } catch (ValidationException $e) {
            $errors = $e->getErrors();
            $this->assertArrayHasKey('password', $errors);
            $this->assertGreaterThan(0, count($errors['password']));
            throw $e;
        }
    }

    public function testRegisterThrowsForDuplicateEmail(): void
    {
        $this->expectException(ValidationException::class);

        $this->userRepository->expects($this->once())
            ->method('existsByEmail')
            ->with('existing@example.com')
            ->willReturn(true);

        $this->authService->register('existing@example.com', 'SecurePass123', 'Test User', '192.168.1.1', 'PHPUnit');
    }

    public function testLoginSucceedsWithValidCredentials(): void
    {
        $user = [
            'id' => 456,
            'email' => 'user@example.com',
            'password_hash' => password_hash('CorrectPassword123', PASSWORD_BCRYPT),
            'full_name' => 'Test User',
            'role' => 'admin',
            'is_active' => true,
            'last_login_at' => null,
        ];

        $this->userRepository->expects($this->once())
            ->method('findByEmail')
            ->with('user@example.com')
            ->willReturn($user);

        $this->userRepository->expects($this->once())
            ->method('updateLastLogin')
            ->with(456);

        $this->jwtService->expects($this->once())
            ->method('generateAccessToken')
            ->with(456, 'user@example.com', 'admin')
            ->willReturn('access-token');

        $this->jwtService->expects($this->once())
            ->method('generateRefreshToken')
            ->willReturn('refresh-token');

        $this->sessionManager->expects($this->once())
            ->method('resetLoginAttempts')
            ->with('192.168.1.1');

        $result = $this->authService->login('user@example.com', 'CorrectPassword123', '192.168.1.1', 'PHPUnit');

        $this->assertArrayHasKey('access_token', $result);
        $this->assertArrayHasKey('refresh_token', $result);
        $this->assertSame('access-token', $result['access_token']);
        $this->assertSame(456, $result['user']['id']);
    }

    public function testLoginThrowsForNonExistentEmail(): void
    {
        $this->expectException(AuthenticationException::class);
        $this->expectExceptionMessage('Invalid email or password');

        $this->userRepository->expects($this->once())
            ->method('findByEmail')
            ->with('nonexistent@example.com')
            ->willReturn(null);

        $this->sessionManager->expects($this->once())
            ->method('incrementLoginAttempts')
            ->with('192.168.1.1');

        $this->authService->login('nonexistent@example.com', 'Password123', '192.168.1.1', 'PHPUnit');
    }

    public function testLoginThrowsForWrongPassword(): void
    {
        $this->expectException(AuthenticationException::class);
        $this->expectExceptionMessage('Invalid email or password');

        $user = [
            'id' => 456,
            'email' => 'user@example.com',
            'password_hash' => password_hash('CorrectPassword123', PASSWORD_BCRYPT),
            'full_name' => 'Test User',
            'role' => 'user',
            'is_active' => true,
        ];

        $this->userRepository->expects($this->once())
            ->method('findByEmail')
            ->with('user@example.com')
            ->willReturn($user);

        $this->sessionManager->expects($this->once())
            ->method('incrementLoginAttempts')
            ->with('192.168.1.1');

        $this->authService->login('user@example.com', 'WrongPassword', '192.168.1.1', 'PHPUnit');
    }

    public function testLoginThrowsForInactiveUser(): void
    {
        $this->expectException(AuthenticationException::class);
        $this->expectExceptionMessage('User account is disabled');

        $user = [
            'id' => 456,
            'email' => 'user@example.com',
            'password_hash' => password_hash('CorrectPassword123', PASSWORD_BCRYPT),
            'full_name' => 'Test User',
            'role' => 'user',
            'is_active' => false,
        ];

        $this->userRepository->expects($this->once())
            ->method('findByEmail')
            ->willReturn($user);

        $this->authService->login('user@example.com', 'CorrectPassword123', '192.168.1.1', 'PHPUnit');
    }

    public function testRefreshSucceedsWithValidToken(): void
    {
        $user = [
            'id' => 789,
            'email' => 'refresh@example.com',
            'role' => 'user',
            'is_active' => true,
        ];

        $this->sessionManager->expects($this->once())
            ->method('validateRefreshToken')
            ->with('valid-refresh-token')
            ->willReturn(789);

        $this->userRepository->expects($this->once())
            ->method('findById')
            ->with(789)
            ->willReturn($user);

        $this->jwtService->expects($this->once())
            ->method('generateAccessToken')
            ->with(789, 'refresh@example.com', 'user')
            ->willReturn('new-access-token');

        $result = $this->authService->refresh('valid-refresh-token', '192.168.1.1', 'PHPUnit');

        $this->assertArrayHasKey('access_token', $result);
        $this->assertSame('new-access-token', $result['access_token']);
    }

    public function testRefreshThrowsForInvalidToken(): void
    {
        $this->expectException(AuthenticationException::class);
        $this->expectExceptionMessage('Invalid or revoked refresh token');

        $this->sessionManager->expects($this->once())
            ->method('validateRefreshToken')
            ->with('invalid-refresh-token')
            ->willReturn(null);

        $this->authService->refresh('invalid-refresh-token', '192.168.1.1', 'PHPUnit');
    }

    public function testLogoutRevokesRefreshToken(): void
    {
        $this->sessionManager->expects($this->once())
            ->method('revokeRefreshToken')
            ->with('refresh-token-to-revoke');

        $this->authService->logout('refresh-token-to-revoke');

        // No exception means success
        $this->assertTrue(true);
    }

    public function testGetCurrentUserReturnsUserData(): void
    {
        $user = [
            'id' => 999,
            'email' => 'current@example.com',
            'full_name' => 'Current User',
            'role' => 'admin',
            'password_hash' => 'should-be-removed',
        ];

        $this->userRepository->expects($this->once())
            ->method('findById')
            ->with(999)
            ->willReturn($user);

        $result = $this->authService->getCurrentUser(999);

        $this->assertArrayHasKey('id', $result);
        $this->assertArrayHasKey('email', $result);
        $this->assertArrayNotHasKey('password_hash', $result); // Sensitive data removed
        $this->assertSame(999, $result['id']);
    }

    public function testGetCurrentUserThrowsForNonExistentUser(): void
    {
        $this->expectException(AuthenticationException::class);
        $this->expectExceptionMessage('User not found');

        $this->userRepository->expects($this->once())
            ->method('findById')
            ->with(999)
            ->willReturn(null);

        $this->authService->getCurrentUser(999);
    }
}
