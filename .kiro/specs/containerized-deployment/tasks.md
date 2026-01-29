# Implementation Plan: Containerized Deployment

## Overview

This plan implements containerized deployment for the Telegram fuel expense bot using a multi-stage Dockerfile and Podman pod specification. The implementation focuses on creating production-ready configuration files with minimal user configuration requirements (only TELEGRAM_TOKEN needed).

## Tasks

- [ ] 1. Update bot configuration for containerized deployment
  - Modify src/config.rs to use hardcoded localhost database defaults for containerized deployment
  - Add environment variable fallbacks: DB_HOST defaults to "localhost", DB_PORT to 3306, DB_USERNAME to "fuel_bot", DB_PASSWORD to "fuel_bot_internal_pass", DB_DATABASE to "fuel_expense_bot"
  - Ensure TELEGRAM_TOKEN remains required with clear error message when missing
  - _Requirements: 2.8, 8.1, 8.2, 8.4_

- [ ]* 1.1 Write validation test for configuration defaults
  - Test that bot starts with only TELEGRAM_TOKEN provided
  - Test that bot fails with clear error when TELEGRAM_TOKEN is missing
  - _Requirements: 8.1, 8.4_

- [ ] 2. Create multi-stage Dockerfile
  - [ ] 2.1 Implement build stage
    - Use rust:1.75-bookworm (or latest stable) as base image with specific version tag
    - Set working directory to /build
    - Copy Cargo.toml and Cargo.lock first for dependency caching
    - Run cargo fetch to download dependencies
    - Copy src/ directory
    - Build with --release flag and strip debug symbols
    - _Requirements: 1.1, 1.3, 9.2_

  - [ ] 2.2 Implement runtime stage
    - Use gcr.io/distroless/cc-debian12:latest as minimal base image
    - Copy compiled binary from build stage to /usr/local/bin/telegram-fuel-bot
    - Create non-root user (UID 1000, username: botuser) - Note: distroless already runs as non-root
    - Set WORKDIR to /app
    - Configure STOPSIGNAL SIGTERM for graceful shutdown
    - Set ENTRYPOINT to ["/usr/local/bin/telegram-fuel-bot"]
    - _Requirements: 1.2, 1.4, 1.5, 5.2, 9.1_

  - [ ]* 2.3 Write Dockerfile validation tests
    - Parse Dockerfile and verify multi-stage structure (two FROM statements)
    - Verify build stage uses rust: base image with version tag
    - Verify runtime stage uses gcr.io/distroless/cc-debian12 base image
    - Verify distroless image runs as non-root by default
    - Verify STOPSIGNAL is set to SIGTERM
    - Verify ENTRYPOINT is configured
    - _Requirements: 1.1, 1.2, 1.5, 5.2, 9.1, 9.2_

  - [ ]* 2.4 Write image build and size validation test
    - Build the Dockerfile
    - Verify build succeeds without errors
    - Check image size is under 50MB (or document if larger)
    - Verify image contains the telegram-fuel-bot binary
    - _Requirements: 1.3, 1.6, 9.3_

- [ ] 3. Create Podman pod specification (pod.yaml)
  - [ ] 3.1 Define pod metadata and structure
    - Create Kubernetes-compatible YAML (apiVersion: v1, kind: Pod)
    - Set pod name to fuel-bot-pod
    - Add labels: app=fuel-expense-bot
    - Configure restartPolicy: Always for automatic recovery
    - Set terminationGracePeriodSeconds: 30 for graceful shutdown
    - _Requirements: 2.1, 5.3, 9.4_

  - [ ] 3.2 Define MariaDB database container
    - Container name: fuel-bot-db
    - Image: docker.io/library/mariadb:11.2 (specific version tag)
    - Environment variables: MARIADB_ROOT_PASSWORD=root_internal_pass, MARIADB_DATABASE=fuel_expense_bot, MARIADB_USER=fuel_bot, MARIADB_PASSWORD=fuel_bot_internal_pass
    - Volume mount: fuel-bot-data to /var/lib/mysql
    - Volume mount: ./scripts to /docker-entrypoint-initdb.d (for initdb.sql)
    - Health check: command ["mariadb", "-e", "SELECT 1"], interval 30s, timeout 5s, retries 3
    - No ports exposed
    - _Requirements: 2.1, 2.3, 2.5, 2.6, 2.9, 3.2, 4.1, 4.4, 4.5, 9.2_

  - [ ] 3.3 Define bot container
    - Container name: fuel-bot-app
    - Image: localhost/fuel-bot:latest
    - Environment variables: TELEGRAM_TOKEN (valueFrom or placeholder), DEFAULT_LIMIT=210.00 (optional), RUST_LOG=telegram_fuel_bot=info (optional)
    - Health check: command ["/usr/local/bin/telegram-fuel-bot", "--version"] or simple exec check (distroless doesn't include shell utilities)
    - Alternative: Use startupProbe with tcpSocket or httpGet if bot exposes health endpoint, or rely on process liveness without explicit health check
    - No ports exposed
    - Depends on database (use initContainers or startup probe to ensure database is ready)
    - _Requirements: 2.1, 2.4, 2.7, 2.10, 4.2, 4.4, 4.5, 9.5_

  - [ ] 3.4 Define volumes
    - Create PersistentVolumeClaim named fuel-bot-pvc for database data
    - Storage: 512Mi (sufficient for expense tracking data)
    - AccessMode: ReadWriteOnce
    - _Requirements: 2.5_

  - [ ]* 3.5 Write pod YAML validation tests
    - Parse pod.yaml and verify structure is valid Kubernetes YAML
    - Verify exactly two containers are defined (database and bot)
    - Verify no external ports are exposed in either container
    - Verify database volume is mounted to /var/lib/mysql
    - Verify scripts volume is mounted to /docker-entrypoint-initdb.d
    - Verify TELEGRAM_TOKEN is in bot container environment
    - Verify database environment variables match bot's hardcoded defaults
    - Verify health checks are configured for both containers
    - Verify terminationGracePeriodSeconds is set
    - Verify restartPolicy is configured
    - _Requirements: 2.1, 2.3, 2.4, 2.5, 2.6, 2.7, 2.9, 4.1, 4.2, 4.4, 4.5, 5.3, 9.4_

- [ ] 4. Create deployment helper script
  - [ ] 4.1 Create scripts/operations/bot.sh script with multiple commands
    - Implement "build" command: podman build -t fuel-bot:latest -f Dockerfile .
    - Implement "deploy" command: podman play kube --replace pod.yaml (check TELEGRAM_TOKEN is set)
    - Implement "stop" command: podman pod stop fuel-bot-pod && podman pod rm fuel-bot-pod
    - Implement "logs" command: podman logs -f fuel-bot-pod-fuel-bot-app (with option for database logs)
    - Implement "status" command: podman pod ps --filter name=fuel-bot-pod
    - Add usage help when no command or invalid command provided
    - Add error handling for each command
    - Make executable (chmod +x)
    - _Requirements: 1.1, 1.2, 2.1, 5.1, 5.3, 6.1, 6.3, 8.4_

- [ ] 5. Create deployment documentation
  - [ ] 5.1 Create DEPLOYMENT.md
    - Document prerequisites (Podman installed, bot token from @BotFather)
    - Document build process: ./scripts/operations/bot.sh build
    - Document deployment process: TELEGRAM_TOKEN=your_token ./scripts/operations/bot.sh deploy
    - Document how to check logs: ./scripts/operations/bot.sh logs
    - Document how to check status: ./scripts/operations/bot.sh status
    - Document how to stop: ./scripts/operations/bot.sh stop
    - Document optional environment variables (DEFAULT_LIMIT, RUST_LOG)
    - Document troubleshooting common issues (database connection, missing token, volume permissions)
    - Include example pod.yaml with placeholder for TELEGRAM_TOKEN
    - _Requirements: 8.5_

  - [ ] 5.2 Update main README.md
    - Add section on containerized deployment
    - Link to DEPLOYMENT.md for detailed instructions
    - Mention minimal configuration requirement (only TELEGRAM_TOKEN)
    - Show quick start example: ./scripts/operations/bot.sh build && TELEGRAM_TOKEN=xxx ./scripts/operations/bot.sh deploy
    - _Requirements: 8.5_

- [ ] 6. Checkpoint - Verify deployment works end-to-end
  - Build the Docker image using ./scripts/operations/bot.sh build
  - Deploy the pod using TELEGRAM_TOKEN=test_token ./scripts/operations/bot.sh deploy
  - Check status using ./scripts/operations/bot.sh status
  - Verify both containers start successfully
  - Verify health checks pass
  - Verify bot connects to database
  - Verify database tables are created (config and counts)
  - Test bot responds to Telegram commands
  - Check logs using ./scripts/operations/bot.sh logs
  - Stop the pod using ./scripts/operations/bot.sh stop
  - Ensure all tests pass, ask the user if questions arise.

- [ ]* 7. Write integration tests (optional)
  - [ ]* 7.1 Write deployment integration test
    - Test that pod deploys successfully with valid TELEGRAM_TOKEN
    - Test that both containers start and become healthy
    - Test that bot can connect to database at localhost:3306
    - _Requirements: 2.2, 4.3_

  - [ ]* 7.2 Write database initialization test
    - Deploy fresh pod
    - Verify database tables (config, counts) are created
    - Verify initialization script ran successfully
    - _Requirements: 3.1, 3.3_

  - [ ]* 7.3 Write persistence test
    - Deploy pod and insert test data
    - Stop and remove pod
    - Redeploy pod
    - Verify test data persists in database
    - _Requirements: 2.5, 2.6_

  - [ ]* 7.4 Write graceful shutdown test
    - Deploy pod
    - Send SIGTERM to bot container
    - Verify bot logs show graceful shutdown message
    - Verify bot stops within terminationGracePeriodSeconds
    - _Requirements: 5.1, 5.2, 5.3_

  - [ ]* 7.5 Write logging test
    - Deploy pod
    - Verify bot logs appear in podman logs output
    - Verify logs include timestamps and log levels
    - Test different RUST_LOG values
    - _Requirements: 6.1, 6.2, 6.4_

  - [ ]* 7.6 Write error handling test
    - Test deployment without TELEGRAM_TOKEN fails with clear error
    - Test bot fails gracefully when database is unavailable
    - _Requirements: 8.4_

- [ ] 8. Optional: Add resource limits to pod specification
  - Add resources section to both containers in pod.yaml
  - Set memory limits: database 512Mi, bot 256Mi
  - Set CPU limits: database 1000m, bot 500m
  - Document how to adjust these limits
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [ ] 9. Final checkpoint - Complete deployment verification
  - Verify all configuration files are correct
  - Verify documentation is complete and accurate
  - Verify deployment works from scratch following documentation
  - Verify all helper scripts work correctly
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster deployment
- The bot's configuration code (src/config.rs) needs to be updated to support hardcoded defaults for containerized deployment
- The existing Dockerfile and docker-compose.yml are outdated (Node.js-based) and will be replaced
- Database credentials are internal to the pod and don't need to be user-configurable
- No external ports are exposed since the bot uses polling mode
- Volume persistence ensures database data survives pod restarts
