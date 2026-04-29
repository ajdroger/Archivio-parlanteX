<?php

declare(strict_types=1);

namespace ArchivioParlante\Repository;

use PDO;
use PDOException;
use Psr\Log\LoggerInterface;

/**
 * Repository for ap_users table CRUD operations
 *
 * All queries use prepared statements for SQL injection prevention.
 * Implements soft delete awareness (WHERE deleted_at IS NULL).
 */
class UserRepository
{
    public function __construct(
        private PDO $pdo,
        private LoggerInterface $logger
    ) {
    }

    /**
     * Find user by email (active users only)
     *
     * @param string $email User email address
     * @return array<string, mixed>|null User data or null if not found
     * @throws \RuntimeException If database query fails
     */
    public function findByEmail(string $email): ?array
    {
        try {
            $stmt = $this->pdo->prepare('
                SELECT id, email, password_hash, full_name, role, is_active, last_login_at, created_at, updated_at
                FROM ap_users
                WHERE email = :email AND deleted_at IS NULL
                LIMIT 1
            ');

            $stmt->execute(['email' => $email]);
            $user = $stmt->fetch(PDO::FETCH_ASSOC);

            return $user !== false ? $user : null;
        } catch (PDOException $e) {
            $this->logger->error('Failed to find user by email', [
                'error' => $e->getMessage(),
                'email' => $email,
            ]);

            throw new \RuntimeException('Database query failed', 0, $e);
        }
    }

    /**
     * Find user by ID (active users only)
     *
     * @param int $id User ID
     * @return array<string, mixed>|null User data or null if not found
     * @throws \RuntimeException If database query fails
     */
    public function findById(int $id): ?array
    {
        try {
            $stmt = $this->pdo->prepare('
                SELECT id, email, password_hash, full_name, role, is_active, last_login_at, created_at, updated_at
                FROM ap_users
                WHERE id = :id AND deleted_at IS NULL
                LIMIT 1
            ');

            $stmt->execute(['id' => $id]);
            $user = $stmt->fetch(PDO::FETCH_ASSOC);

            return $user !== false ? $user : null;
        } catch (PDOException $e) {
            $this->logger->error('Failed to find user by ID', [
                'error' => $e->getMessage(),
                'user_id' => $id,
            ]);

            throw new \RuntimeException('Database query failed', 0, $e);
        }
    }

    /**
     * Create new user
     *
     * @param string $email User email (must be unique)
     * @param string $passwordHash Bcrypt hashed password (use password_hash())
     * @param string $fullName User full name
     * @param string $role User role (default: 'user')
     * @return int Created user ID
     * @throws \RuntimeException If database insert fails or email is duplicate
     */
    public function create(string $email, string $passwordHash, string $fullName, string $role = 'user'): int
    {
        try {
            $stmt = $this->pdo->prepare('
                INSERT INTO ap_users (email, password_hash, full_name, role, is_active, created_at, updated_at)
                VALUES (:email, :password_hash, :full_name, :role, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ');

            $stmt->execute([
                'email' => $email,
                'password_hash' => $passwordHash,
                'full_name' => $fullName,
                'role' => $role,
            ]);

            $userId = (int) $this->pdo->lastInsertId();

            $this->logger->info('User created successfully', [
                'user_id' => $userId,
                'email' => $email,
                'role' => $role,
            ]);

            return $userId;
        } catch (PDOException $e) {
            // Check for duplicate email (MySQL error code 1062 = ER_DUP_ENTRY)
            if ($e->getCode() === '23000') {
                $this->logger->warning('Attempted to create user with duplicate email', [
                    'email' => $email,
                ]);

                throw new \RuntimeException('Email already exists', 0, $e);
            }

            $this->logger->error('Failed to create user', [
                'error' => $e->getMessage(),
                'email' => $email,
            ]);

            throw new \RuntimeException('Database insert failed', 0, $e);
        }
    }

    /**
     * Update last login timestamp for user
     *
     * @param int $userId User ID
     * @return void
     * @throws \RuntimeException If database update fails
     */
    public function updateLastLogin(int $userId): void
    {
        try {
            $stmt = $this->pdo->prepare('
                UPDATE ap_users
                SET last_login_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
                WHERE id = :user_id AND deleted_at IS NULL
            ');

            $stmt->execute(['user_id' => $userId]);

            $this->logger->debug('Updated last login timestamp', ['user_id' => $userId]);
        } catch (PDOException $e) {
            $this->logger->error('Failed to update last login', [
                'error' => $e->getMessage(),
                'user_id' => $userId,
            ]);

            throw new \RuntimeException('Database update failed', 0, $e);
        }
    }

    /**
     * Check if user with email exists
     *
     * @param string $email User email address
     * @return bool True if user exists (not deleted), false otherwise
     */
    public function existsByEmail(string $email): bool
    {
        try {
            $stmt = $this->pdo->prepare('
                SELECT COUNT(*) as count
                FROM ap_users
                WHERE email = :email AND deleted_at IS NULL
            ');

            $stmt->execute(['email' => $email]);
            $result = $stmt->fetch(PDO::FETCH_ASSOC);

            return $result !== false && $result['count'] > 0;
        } catch (PDOException $e) {
            $this->logger->error('Failed to check email existence', [
                'error' => $e->getMessage(),
                'email' => $email,
            ]);

            return false;
        }
    }
}
