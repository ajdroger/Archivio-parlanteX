<?php

declare(strict_types=1);

use DI\Container;
use Monolog\Handler\StreamHandler;
use Monolog\Logger;
use Psr\Log\LoggerInterface;

return function (): Container {
    $builder = new \DI\ContainerBuilder();
    
    $builder->addDefinitions([
        // Logger
        LoggerInterface::class => function () {
            $logger = new Logger('archivio-parlante');
            $logger->pushHandler(
                new StreamHandler('php://stdout', Logger::DEBUG)
            );
            return $logger;
        },

        // Guzzle HTTP Client
        \GuzzleHttp\Client::class => function () {
            return new \GuzzleHttp\Client([
                'timeout' => 30.0,
                'http_errors' => false, // Handle errors manually
            ]);
        },

        // PDO MySQL Connection
        \PDO::class => function () {
            $host = $_ENV['MYSQL_HOST'] ?? 'mysql';
            $db = $_ENV['MYSQL_DB'] ?? 'archivio_parlante_x';
            $user = $_ENV['MYSQL_USER'] ?? 'root';
            $pass = $_ENV['MYSQL_PASSWORD'] ?? '';
            $dsn = "mysql:host={$host};dbname={$db};charset=utf8mb4";

            return new \PDO($dsn, $user, $pass, [
                \PDO::ATTR_ERRMODE => \PDO::ERRMODE_EXCEPTION,
                \PDO::ATTR_DEFAULT_FETCH_MODE => \PDO::FETCH_ASSOC,
                \PDO::ATTR_EMULATE_PREPARES => false,
            ]);
        },

        // Rust Engine Proxy Service
        \ArchivioParlante\Service\RustEngineProxy::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Service\RustEngineProxy(
                $c->get(\GuzzleHttp\Client::class),
                $c->get(LoggerInterface::class),
                $_ENV['RUST_ENGINE_URL'] ?? 'http://rust-engine:8090',
                $_ENV['RUST_ENGINE_INTERNAL_TOKEN'] ?? ''
            );
        },

        // Redis Client (Predis)
        \Predis\Client::class => function () {
            $redisUrl = $_ENV['REDIS_URL'] ?? 'redis://redis:6379';
            return new \Predis\Client($redisUrl);
        },

        // Repositories
        \ArchivioParlante\Repository\UserRepository::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Repository\UserRepository(
                $c->get(\PDO::class),
                $c->get(LoggerInterface::class)
            );
        },

        \ArchivioParlante\Repository\AuditLogRepository::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Repository\AuditLogRepository(
                $c->get(\PDO::class),
                $c->get(LoggerInterface::class)
            );
        },

        // Services
        \ArchivioParlante\Service\JwtService::class => function (\Psr\Container\ContainerInterface $c) {
            $secret = $_ENV['JWT_SECRET'] ?? throw new \RuntimeException('JWT_SECRET not configured');
            return new \ArchivioParlante\Service\JwtService(
                $secret,
                $c->get(LoggerInterface::class)
            );
        },

        \ArchivioParlante\Service\RedisSessionManager::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Service\RedisSessionManager(
                $c->get(\Predis\Client::class),
                $c->get(LoggerInterface::class)
            );
        },

        \ArchivioParlante\Service\AuditLogger::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Service\AuditLogger(
                $c->get(\ArchivioParlante\Repository\AuditLogRepository::class),
                $c->get(LoggerInterface::class)
            );
        },

        \ArchivioParlante\Service\AuthService::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Service\AuthService(
                $c->get(\ArchivioParlante\Repository\UserRepository::class),
                $c->get(\ArchivioParlante\Service\JwtService::class),
                $c->get(\ArchivioParlante\Service\RedisSessionManager::class),
                $c->get(\ArchivioParlante\Service\AuditLogger::class),
                $c->get(LoggerInterface::class)
            );
        },

        \ArchivioParlante\Service\WorkspaceService::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Service\WorkspaceService(
                $c->get(\PDO::class),
                $c->get(LoggerInterface::class)
            );
        },

        // Middleware
        \ArchivioParlante\Middleware\AuthMiddleware::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Middleware\AuthMiddleware(
                $c->get(\ArchivioParlante\Service\JwtService::class),
                $c->get(\ArchivioParlante\Service\AuditLogger::class),
                $c->get(LoggerInterface::class)
            );
        },

        // Controllers
        \ArchivioParlante\Controller\HealthController::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Controller\HealthController(
                $c->get(LoggerInterface::class),
                $c->get(\ArchivioParlante\Service\RustEngineProxy::class)
            );
        },

        \ArchivioParlante\Controller\AuthController::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Controller\AuthController(
                $c->get(\ArchivioParlante\Service\AuthService::class),
                $c->get(LoggerInterface::class)
            );
        },

        \ArchivioParlante\Controller\ProxyController::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Controller\ProxyController(
                $c->get(\ArchivioParlante\Service\RustEngineProxy::class),
                $c->get(\ArchivioParlante\Service\AuditLogger::class),
                $c->get(LoggerInterface::class)
            );
        },

        \ArchivioParlante\Controller\WorkspaceController::class => function (\Psr\Container\ContainerInterface $c) {
            return new \ArchivioParlante\Controller\WorkspaceController(
                $c->get(\ArchivioParlante\Service\WorkspaceService::class),
                $c->get(LoggerInterface::class)
            );
        },
    ]);

    return $builder->build();
};
