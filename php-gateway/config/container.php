<?php

declare(strict_types=1);

use DI\Container;
use Monolog\Handler\StreamHandler;
use Monolog\Logger;
use Psr\Log\LoggerInterface;

return function (): Container {
    $container = new Container();

    // Logger
    $container->set(LoggerInterface::class, function () {
        $logger = new Logger('archivio-parlante');
        $logger->pushHandler(
            new StreamHandler('php://stdout', Logger::DEBUG)
        );
        return $logger;
    });

    // Guzzle HTTP Client
    $container->set(\GuzzleHttp\Client::class, function () {
        return new \GuzzleHttp\Client([
            'timeout' => 30.0,
            'http_errors' => false, // Handle errors manually
        ]);
    });

    // Rust Engine Proxy Service
    $container->set(\ArchivioParlante\Service\RustEngineProxy::class, function ($c) {
        return new \ArchivioParlante\Service\RustEngineProxy(
            $c->get(\GuzzleHttp\Client::class),
            $c->get(LoggerInterface::class),
            $_ENV['RUST_ENGINE_URL'],
            $_ENV['RUST_ENGINE_INTERNAL_TOKEN'] ?? ''
        );
    });

    // === Fase 3.2: Authentication Services ===

    // PDO MySQL Connection
    $container->set(\PDO::class, function () {
        $host = $_ENV['MYSQL_HOST'] ?? 'mysql';
        $port = $_ENV['MYSQL_PORT'] ?? '3306';
        $db = $_ENV['MYSQL_DB'] ?? 'archivio_parlante_x';
        $user = $_ENV['MYSQL_USER'] ?? 'root';
        $password = $_ENV['MYSQL_PASSWORD'] ?? '';

        $dsn = sprintf('mysql:host=%s;port=%s;dbname=%s;charset=utf8mb4', $host, $port, $db);

        $pdo = new \PDO($dsn, $user, $password, [
            \PDO::ATTR_ERRMODE => \PDO::ERRMODE_EXCEPTION,
            \PDO::ATTR_DEFAULT_FETCH_MODE => \PDO::FETCH_ASSOC,
            \PDO::ATTR_EMULATE_PREPARES => false,
        ]);

        return $pdo;
    });

    // Redis Client (Predis)
    $container->set(\Predis\Client::class, function () {
        $redisUrl = $_ENV['REDIS_URL'] ?? 'redis://redis:6379';
        return new \Predis\Client($redisUrl);
    });

    // User Repository
    $container->set(\ArchivioParlante\Repository\UserRepository::class, function ($c) {
        return new \ArchivioParlante\Repository\UserRepository(
            $c->get(\PDO::class),
            $c->get(LoggerInterface::class)
        );
    });

    // Redis Session Manager
    $container->set(\ArchivioParlante\Service\RedisSessionManager::class, function ($c) {
        return new \ArchivioParlante\Service\RedisSessionManager(
            $c->get(\Predis\Client::class),
            $c->get(LoggerInterface::class)
        );
    });

    // JWT Service
    $container->set(\ArchivioParlante\Service\JwtService::class, function ($c) {
        $jwtSecret = $_ENV['JWT_SECRET'] ?? throw new \RuntimeException('JWT_SECRET not configured in .env');
        return new \ArchivioParlante\Service\JwtService(
            $jwtSecret,
            $c->get(LoggerInterface::class)
        );
    });

    // Auth Service
    $container->set(\ArchivioParlante\Service\AuthService::class, function ($c) {
        return new \ArchivioParlante\Service\AuthService(
            $c->get(\ArchivioParlante\Repository\UserRepository::class),
            $c->get(\ArchivioParlante\Service\JwtService::class),
            $c->get(\ArchivioParlante\Service\RedisSessionManager::class),
            $c->get(LoggerInterface::class)
        );
    });

    // Auth Controller
    $container->set(\ArchivioParlante\Controller\AuthController::class, function ($c) {
        return new \ArchivioParlante\Controller\AuthController(
            $c->get(\ArchivioParlante\Service\AuthService::class),
            $c->get(LoggerInterface::class)
        );
    });

    // Auth Middleware
    $container->set(\ArchivioParlante\Middleware\AuthMiddleware::class, function ($c) {
        return new \ArchivioParlante\Middleware\AuthMiddleware(
            $c->get(\ArchivioParlante\Service\JwtService::class),
            $c->get(LoggerInterface::class)
        );
    });

    // Rate Limit Middleware
    $container->set(\ArchivioParlante\Middleware\RateLimitMiddleware::class, function ($c) {
        return new \ArchivioParlante\Middleware\RateLimitMiddleware(
            $c->get(\ArchivioParlante\Service\RedisSessionManager::class),
            $c->get(LoggerInterface::class)
        );
    });

    return $container;
};
