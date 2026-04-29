<?php

declare(strict_types=1);

namespace ArchivioParlante\Service;

use Predis\Client as RedisClient;
use Predis\PredisException;
use Psr\Log\LoggerInterface;

/**
 * Redis-based session and refresh token manager
 *
 * Manages:
 * - Refresh tokens with TTL (7 days default)
 * - User session activity tracking
 * - Token revocation on logout
 *
 * Redis key patterns:
 * - refresh:{token_hash} => user_id (TTL: 7 days)
 * - session:{user_id} => last_activity_timestamp
 * - ratelimit:login:{ip} => attempt_count (TTL: 15 min)
 */
class RedisSessionManager
{
    private const REFRESH_TOKEN_TTL = 604800; // 7 days in seconds
    private const RATE_LIMIT_TTL = 900; // 15 minutes in seconds

    public function __construct(
        private RedisClient $redis,
        private LoggerInterface $logger
    ) {
    }

    /**
     * Store refresh token in Redis with TTL
     *
     * @param string $token Refresh token (64-byte hex string)
     * @param int $userId User ID owning the token
     * @param int $ttl Time to live in seconds (default: 7 days)
     * @return void
     */
    public function storeRefreshToken(string $token, int $userId, int $ttl = self::REFRESH_TOKEN_TTL): void
    {
        try {
            $key = $this->getRefreshTokenKey($token);
            $this->redis->setex($key, $ttl, (string) $userId);

            $this->logger->debug('Stored refresh token', [
                'user_id' => $userId,
                'ttl' => $ttl,
            ]);
        } catch (PredisException $e) {
            $this->logger->error('Failed to store refresh token', [
                'error' => $e->getMessage(),
                'user_id' => $userId,
            ]);

            throw new \RuntimeException('Redis operation failed', 0, $e);
        }
    }

    /**
     * Validate refresh token and return user ID
     *
     * @param string $token Refresh token to validate
     * @return int|null User ID if valid, null if token not found or expired
     */
    public function validateRefreshToken(string $token): ?int
    {
        try {
            $key = $this->getRefreshTokenKey($token);
            $userId = $this->redis->get($key);

            if ($userId === null) {
                $this->logger->debug('Refresh token not found or expired');
                return null;
            }

            $this->logger->debug('Refresh token validated', ['user_id' => $userId]);

            return (int) $userId;
        } catch (PredisException $e) {
            $this->logger->error('Failed to validate refresh token', [
                'error' => $e->getMessage(),
            ]);

            return null;
        }
    }

    /**
     * Revoke refresh token (logout)
     *
     * @param string $token Refresh token to revoke
     * @return void
     */
    public function revokeRefreshToken(string $token): void
    {
        try {
            $key = $this->getRefreshTokenKey($token);
            $deleted = $this->redis->del([$key]);

            if ($deleted > 0) {
                $this->logger->info('Refresh token revoked', ['key' => $key]);
            } else {
                $this->logger->debug('Refresh token already expired or not found');
            }
        } catch (PredisException $e) {
            $this->logger->error('Failed to revoke refresh token', [
                'error' => $e->getMessage(),
            ]);

            throw new \RuntimeException('Redis operation failed', 0, $e);
        }
    }

    /**
     * Update user session activity timestamp
     *
     * @param int $userId User ID
     * @return void
     */
    public function updateSessionActivity(int $userId): void
    {
        try {
            $key = $this->getSessionKey($userId);
            $this->redis->set($key, (string) time());

            $this->logger->debug('Updated session activity', ['user_id' => $userId]);
        } catch (PredisException $e) {
            $this->logger->error('Failed to update session activity', [
                'error' => $e->getMessage(),
                'user_id' => $userId,
            ]);

            // Non-critical, don't throw
        }
    }

    /**
     * Revoke all sessions for a user (e.g., on password change)
     *
     * @param int $userId User ID
     * @return void
     */
    public function revokeAllUserSessions(int $userId): void
    {
        try {
            // Delete session activity
            $sessionKey = $this->getSessionKey($userId);
            $this->redis->del([$sessionKey]);

            // Scan for all refresh tokens belonging to this user
            // Note: This is expensive, consider storing token list in a set
            $pattern = 'refresh:*';
            $cursor = 0;
            $revokedCount = 0;

            do {
                $result = $this->redis->scan($cursor, ['MATCH' => $pattern, 'COUNT' => 100]);
                $cursor = $result[0] ?? 0;
                $keys = $result[1] ?? [];

                foreach ($keys as $key) {
                    $storedUserId = $this->redis->get($key);
                    if ($storedUserId === (string) $userId) {
                        $this->redis->del([$key]);
                        $revokedCount++;
                    }
                }
            } while ($cursor !== 0);

            $this->logger->info('Revoked all user sessions', [
                'user_id' => $userId,
                'tokens_revoked' => $revokedCount,
            ]);
        } catch (PredisException $e) {
            $this->logger->error('Failed to revoke all user sessions', [
                'error' => $e->getMessage(),
                'user_id' => $userId,
            ]);

            throw new \RuntimeException('Redis operation failed', 0, $e);
        }
    }

    /**
     * Generic rate limit counter increment
     *
     * @param string $key Redis key for rate limit counter
     * @param int $ttl Time to live in seconds
     * @return int Number of attempts
     */
    public function incrementRateLimitCounter(string $key, int $ttl): int
    {
        try {
            $attempts = $this->redis->incr($key);

            // Set TTL on first attempt
            if ($attempts === 1) {
                $this->redis->expire($key, $ttl);
            }

            $this->logger->debug('Incremented rate limit counter', [
                'key' => $key,
                'attempts' => $attempts,
                'ttl' => $ttl,
            ]);

            return (int) $attempts;
        } catch (PredisException $e) {
            $this->logger->error('Failed to increment rate limit counter', [
                'error' => $e->getMessage(),
                'key' => $key,
            ]);

            // Return 0 to allow request (fail open on Redis error)
            return 0;
        }
    }

    /**
     * Increment login attempt counter for IP address
     *
     * @param string $ip IP address
     * @return int Number of attempts
     */
    public function incrementLoginAttempts(string $ip): int
    {
        $key = $this->getRateLimitKey($ip);
        return $this->incrementRateLimitCounter($key, self::RATE_LIMIT_TTL);
    }

    /**
     * Generic get rate limit attempts
     *
     * @param string $key Redis key for rate limit counter
     * @return int Number of attempts
     */
    public function getRateLimitAttempts(string $key): int
    {
        try {
            $attempts = $this->redis->get($key);

            return $attempts !== null ? (int) $attempts : 0;
        } catch (PredisException $e) {
            $this->logger->error('Failed to get rate limit attempts', [
                'error' => $e->getMessage(),
                'key' => $key,
            ]);

            return 0;
        }
    }

    /**
     * Get current login attempt count for IP
     *
     * @param string $ip IP address
     * @return int Number of attempts
     */
    public function getLoginAttempts(string $ip): int
    {
        $key = $this->getRateLimitKey($ip);
        return $this->getRateLimitAttempts($key);
    }

    /**
     * Reset login attempts for IP
     *
     * @param string $ip IP address
     * @return void
     */
    public function resetLoginAttempts(string $ip): void
    {
        try {
            $key = $this->getRateLimitKey($ip);
            $this->redis->del([$key]);

            $this->logger->debug('Reset login attempts', ['ip' => $ip]);
        } catch (PredisException $e) {
            $this->logger->error('Failed to reset login attempts', [
                'error' => $e->getMessage(),
                'ip' => $ip,
            ]);

            // Non-critical, don't throw
        }
    }

    /**
     * Get Redis key for refresh token
     *
     * @param string $token Refresh token
     * @return string Redis key
     */
    private function getRefreshTokenKey(string $token): string
    {
        // Hash token to prevent exposure in Redis keyspace
        return 'refresh:' . hash('sha256', $token);
    }

    /**
     * Get Redis key for user session
     *
     * @param int $userId User ID
     * @return string Redis key
     */
    private function getSessionKey(int $userId): string
    {
        return 'session:' . $userId;
    }

    /**
     * Get Redis key for rate limiting
     *
     * @param string $ip IP address
     * @return string Redis key
     */
    private function getRateLimitKey(string $ip): string
    {
        return 'ratelimit:login:' . $ip;
    }
}
