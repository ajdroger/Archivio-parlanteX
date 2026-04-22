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

    return $container;
};
