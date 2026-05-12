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
        }
    ]);

    return $builder->build();
};
