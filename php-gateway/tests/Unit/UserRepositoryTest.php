<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Repository\UserRepository;
use PHPUnit\Framework\TestCase;
use Psr\Log\NullLogger;

final class UserRepositoryTest extends TestCase
{
    private \PDO $pdo;
    private UserRepository $repository;

    protected function setUp(): void
    {
        // Use in-memory SQLite for testing
        $this->pdo = new \PDO('sqlite::memory:');
        $this->pdo->setAttribute(\PDO::ATTR_ERRMODE, \PDO::ERRMODE_EXCEPTION);

        // Create ap_users table
        $this->pdo->exec('
            CREATE TABLE ap_users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                full_name TEXT NOT NULL,
                role TEXT DEFAULT "user",
                is_active INTEGER DEFAULT 1,
                last_login_at TEXT,
                created_at TEXT,
                updated_at TEXT,
                deleted_at TEXT
            )
        ');

        $this->repository = new UserRepository($this->pdo, new NullLogger());
    }

    public function testFindByEmailReturnsUserWhenExists(): void
    {
        $this->pdo->exec("
            INSERT INTO ap_users (email, password_hash, full_name, role)
            VALUES ('test@example.com', 'hash123', 'Test User', 'user')
        ");

        $user = $this->repository->findByEmail('test@example.com');

        $this->assertNotNull($user);
        $this->assertSame('test@example.com', $user['email']);
        $this->assertSame('Test User', $user['full_name']);
    }

    public function testFindByEmailReturnsNullWhenNotExists(): void
    {
        $user = $this->repository->findByEmail('nonexistent@example.com');

        $this->assertNull($user);
    }

    public function testFindByEmailIgnoresDeletedUsers(): void
    {
        $this->pdo->exec("
            INSERT INTO ap_users (email, password_hash, full_name, deleted_at)
            VALUES ('deleted@example.com', 'hash', 'Deleted', '2024-01-01')
        ");

        $user = $this->repository->findByEmail('deleted@example.com');

        $this->assertNull($user);
    }

    public function testCreateSuccessfully(): void
    {
        $userId = $this->repository->create('new@example.com', 'hashed_password', 'New User');

        $this->assertGreaterThan(0, $userId);

        $stmt = $this->pdo->query("SELECT * FROM ap_users WHERE id = $userId");
        $user = $stmt->fetch(\PDO::FETCH_ASSOC);

        $this->assertSame('new@example.com', $user['email']);
        $this->assertSame('hashed_password', $user['password_hash']);
    }

    public function testCreateThrowsForDuplicateEmail(): void
    {
        $this->expectException(\RuntimeException::class);

        $this->repository->create('duplicate@example.com', 'hash1', 'User 1');
        $this->repository->create('duplicate@example.com', 'hash2', 'User 2');
    }

    public function testUpdateLastLogin(): void
    {
        $this->pdo->exec("
            INSERT INTO ap_users (email, password_hash, full_name)
            VALUES ('login@example.com', 'hash', 'Login User')
        ");

        $userId = (int) $this->pdo->lastInsertId();

        $this->repository->updateLastLogin($userId);

        $stmt = $this->pdo->query("SELECT last_login_at FROM ap_users WHERE id = $userId");
        $lastLogin = $stmt->fetchColumn();

        $this->assertNotNull($lastLogin);
    }

    public function testExistsByEmailReturnsTrueWhenExists(): void
    {
        $this->pdo->exec("
            INSERT INTO ap_users (email, password_hash, full_name)
            VALUES ('exists@example.com', 'hash', 'User')
        ");

        $exists = $this->repository->existsByEmail('exists@example.com');

        $this->assertTrue($exists);
    }

    public function testExistsByEmailReturnsFalseWhenNotExists(): void
    {
        $exists = $this->repository->existsByEmail('notexists@example.com');

        $this->assertFalse($exists);
    }
}
