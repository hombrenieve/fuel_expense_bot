#!/bin/bash
# Quick setup script for development database with Podman/Docker

set -e

# Detect if we should use podman or docker
if command -v podman &> /dev/null; then
    CONTAINER_CMD="podman"
elif command -v docker &> /dev/null; then
    CONTAINER_CMD="docker"
else
    echo "Error: Neither podman nor docker found. Please install one of them."
    exit 1
fi

echo "Using: $CONTAINER_CMD"
echo ""

# Configuration
CONTAINER_NAME="fuel_bot_db"
VOLUME_NAME="fuel_bot_data"
DB_ROOT_PASSWORD="rootpass"
DB_NAME="fuel_expense_bot"
DB_USER="fuel_bot"
DB_PASSWORD="botpass"
DB_PORT="3306"

# Check if container already exists
if $CONTAINER_CMD ps -a --format "{{.Names}}" | grep -q "^${CONTAINER_NAME}$"; then
    echo "Container '$CONTAINER_NAME' already exists."
    
    # Check if it's running
    if $CONTAINER_CMD ps --format "{{.Names}}" | grep -q "^${CONTAINER_NAME}$"; then
        echo "Container is already running."
    else
        echo "Starting existing container..."
        $CONTAINER_CMD start $CONTAINER_NAME
        echo "Waiting for database to be ready..."
        sleep 5
    fi
else
    # Create volume if it doesn't exist (idempotent)
    echo "Creating volume '$VOLUME_NAME' (if it doesn't exist)..."
    $CONTAINER_CMD volume create $VOLUME_NAME 2>/dev/null || echo "Volume already exists, continuing..."
    
    echo "Starting MariaDB container..."
    $CONTAINER_CMD run -d \
      --name $CONTAINER_NAME \
      -e MYSQL_ROOT_PASSWORD=$DB_ROOT_PASSWORD \
      -e MYSQL_DATABASE=$DB_NAME \
      -e MYSQL_USER=$DB_USER \
      -e MYSQL_PASSWORD=$DB_PASSWORD \
      -p $DB_PORT:3306 \
      -v $VOLUME_NAME:/var/lib/mysql \
      docker.io/library/mariadb:latest
    
    echo "Waiting for database to be ready..."
    # Wait for MySQL to be ready (up to 60 seconds)
    for i in {1..60}; do
        if $CONTAINER_CMD exec $CONTAINER_NAME mariadb -u$DB_USER -p$DB_PASSWORD -e "SELECT 1" 2>&1 | grep -q "1"; then
            echo ""
            echo "Database is ready!"
            break
        fi
        if [ $i -eq 60 ]; then
            echo ""
            echo "Error: Database did not become ready in time"
            echo "Check logs with: $CONTAINER_CMD logs $CONTAINER_NAME"
            exit 1
        fi
        echo -n "."
        sleep 1
    done
    
    echo "Initializing database schema..."
    $CONTAINER_CMD exec -i $CONTAINER_NAME mariadb -u$DB_USER -p$DB_PASSWORD $DB_NAME < scripts/initdb.sql
    
    echo ""
    echo "Verifying tables..."
    $CONTAINER_CMD exec $CONTAINER_NAME mariadb -u$DB_USER -p$DB_PASSWORD $DB_NAME -e "SHOW TABLES;"
fi

echo ""
echo "✅ Database is ready!"
echo ""
echo "Connection details:"
echo "  Host: localhost"
echo "  Port: $DB_PORT"
echo "  Database: $DB_NAME"
echo "  Username: $DB_USER"
echo "  Password: $DB_PASSWORD"
echo ""
echo "Add these to your .env file:"
echo ""
echo "DB_HOST=localhost"
echo "DB_PORT=$DB_PORT"
echo "DB_USERNAME=$DB_USER"
echo "DB_PASSWORD=$DB_PASSWORD"
echo "DB_DATABASE=$DB_NAME"
echo ""
echo "Don't forget to set TELEGRAM_TOKEN in .env!"
echo ""
echo "To stop the database: $CONTAINER_CMD stop $CONTAINER_NAME"
echo "To remove everything: $CONTAINER_CMD rm -f $CONTAINER_NAME && $CONTAINER_CMD volume rm $VOLUME_NAME"
