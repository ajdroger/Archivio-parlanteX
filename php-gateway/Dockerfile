# ============================================================================
# Archivio Parlante — PHP Gateway Dockerfile
# ============================================================================

FROM php:8.2-apache

# Install system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    git \
    unzip \
    libzip-dev \
    && rm -rf /var/lib/apt/lists/*

# Install PHP extensions
RUN docker-php-ext-install pdo pdo_mysql zip

# Install Redis extension
RUN pecl install redis && \
    docker-php-ext-enable redis

# Install Composer
COPY --from=composer:2 /usr/bin/composer /usr/bin/composer

# Enable Apache mod_rewrite
RUN a2enmod rewrite

# Set working directory
WORKDIR /var/www/html

# Copy composer files
COPY composer.json composer.lock* ./

# Install dependencies
RUN composer install --no-dev --optimize-autoloader --no-interaction

# Copy application code
COPY . .

# Configure Apache VirtualHost (must be after COPY . .)
RUN cp /var/www/html/apache-vhost.conf /etc/apache2/sites-available/000-default.conf && \
    cat /etc/apache2/sites-available/000-default.conf | grep "DocumentRoot"

# Set permissions
RUN chown -R www-data:www-data /var/www/html

# Expose port
EXPOSE 80

# Start Apache
CMD ["apache2-foreground"]
