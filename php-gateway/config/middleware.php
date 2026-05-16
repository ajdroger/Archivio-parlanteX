<?php

declare(strict_types=1);

use Slim\App;

return function (App $app): void {
    // Parse JSON body
    $app->addBodyParsingMiddleware();

    // Add routing middleware
    $app->addRoutingMiddleware();

    // Error middleware
    $errorMiddleware = $app->addErrorMiddleware(
        displayErrorDetails: $_ENV['APP_DEBUG'] === 'true',
        logErrors: true,
        logErrorDetails: true
    );
};
