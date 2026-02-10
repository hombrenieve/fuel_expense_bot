# Containerized Deployment Guide

This guide explains how to deploy the Telegram Fuel Expense Bot using Podman pods.

## Prerequisites

Before deploying, ensure you have:

1. **Podman installed**: Install Podman for your platform
   - Linux: `sudo apt install podman` or `sudo dnf install podman`
   - macOS: `brew install podman` then `podman machine init && podman machine start`
   - Windows: Install via [Podman Desktop](https://podman-desktop.io/)

2. **envsubst utility**: For environment variable substitution (usually pre-installed)
   - Linux: Part of `gettext` package - `sudo apt install gettext-base` or `sudo dnf install gettext`
   - macOS: Pre-installed or `brew install gettext`
   - Windows: Included with Git Bash or WSL

3. **Telegram Bot Token**: Obtain a bot token from [@BotFather](https://t.me/botfather)
   - Start a chat with @BotFather on Telegram
   - Send `/newbot` and follow the prompts
   - Save the token provided (format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

3. **Source code**: Clone this repository and navigate to the project directory

## Deployment Steps

### Step 1: Build the Bot Container Image

Build the bot container image from the Dockerfile:

```bash
podman build -t fuel-bot:latest -f Dockerfile .
```

This creates a minimal container image (~15-50 MB) with the compiled Rust application.

### Step 2: Create the Persistent Volume Claim

Create the persistent volume for database storage:

```bash
podman play kube pvc.yaml
```

This creates a volume named `fuel-bot-pvc` with 512Mi of storage for the MariaDB database.

### Step 3: Deploy the Pod

Deploy the pod with your Telegram bot token. You can pass the token as an environment variable without modifying the pod.yaml file:

```bash
export TELEGRAM_TOKEN="123456789:ABCdefGHIjklMNOpqrsTUVwxyz"
envsubst < pod.yaml | podman play kube --replace -
```

Or in a single command:

```bash
TELEGRAM_TOKEN="123456789:ABCdefGHIjklMNOpqrsTUVwxyz" envsubst < pod.yaml | podman play kube --replace -
```

The `--replace` flag allows redeployment if the pod already exists.

**Alternative method** (if you prefer to modify the file):

Edit `pod.yaml` and replace `YOUR_TELEGRAM_TOKEN_HERE` with your actual token, then deploy:

```bash
podman play kube --replace pod.yaml
```

## Monitoring and Management

### Check Pod Status

View the status of the pod and its containers:

```bash
podman pod ps --filter name=fuel-bot-pod
```

You should see the pod running with 2 containers (bot and database).

### View Logs

Check the bot application logs:

```bash
podman logs -f fuel-bot-pod-fuel-bot-app
```

Check the database logs:

```bash
podman logs -f fuel-bot-pod-fuel-bot-db
```

The `-f` flag follows the logs in real-time. Remove it to view existing logs only.

### Stop the Pod

To stop and remove the pod:

```bash
podman pod stop fuel-bot-pod && podman pod rm fuel-bot-pod
```

Note: The persistent volume remains intact, so your data is preserved.

## Optional Configuration

### Environment Variables

You can customize the bot behavior with optional environment variables in `pod.yaml`:

**DEFAULT_LIMIT**: Set the default monthly spending limit for new users (default: 210.00)
```yaml
- name: DEFAULT_LIMIT
  value: "300.00"
```

**RUST_LOG**: Configure logging verbosity (default: telegram_fuel_bot=info)
```yaml
- name: RUST_LOG
  value: "telegram_fuel_bot=debug"
```

Available log levels: `error`, `warn`, `info`, `debug`, `trace`

## Troubleshooting

### Bot Container Fails to Start

**Symptom**: Bot container repeatedly restarts or shows "CrashLoopBackOff"

**Possible Causes**:

1. **Missing TELEGRAM_TOKEN**
   - Error: "TELEGRAM_TOKEN environment variable is required"
   - Solution: Ensure you've set the TELEGRAM_TOKEN environment variable when deploying:
     ```bash
     export TELEGRAM_TOKEN="your_token"
     envsubst < pod.yaml | podman play kube --replace -
     ```

2. **Invalid TELEGRAM_TOKEN**
   - Error: "Failed to authenticate with Telegram API"
   - Solution: Verify your token is correct and hasn't been revoked

3. **Database not ready**
   - Error: "Failed to connect to database at localhost:3306"
   - Solution: Wait 30-60 seconds for the database to initialize on first run
   - Check database logs: `podman logs fuel-bot-pod-fuel-bot-db`

### Database Connection Issues

**Symptom**: Bot logs show "Failed to connect to database"

**Solutions**:

1. Check if the database container is running:
   ```bash
   podman ps --filter name=fuel-bot-db
   ```

2. Verify database health:
   ```bash
   podman exec fuel-bot-pod-fuel-bot-db mariadb -u fuel_bot -pfuel_bot_internal_pass -e "SELECT 1"
   ```

3. Check database initialization:
   ```bash
   podman exec fuel-bot-pod-fuel-bot-db mariadb -u fuel_bot -pfuel_bot_internal_pass fuel_expense_bot -e "SHOW TABLES"
   ```
   
   You should see `config` and `counts` tables.

### Volume Permission Errors

**Symptom**: Database container fails with "Permission denied" errors

**Solutions**:

1. **SELinux issues** (RHEL/Fedora/CentOS):
   - Add `:Z` suffix to volume mounts in pod.yaml for SELinux relabeling
   - Or temporarily disable SELinux: `sudo setenforce 0`

2. **User namespace issues**:
   - Run with user namespace mapping: `podman play kube --userns=keep-id pod.yaml`

3. **Volume ownership**:
   - Check volume location: `podman volume inspect fuel-bot-pvc`
   - Fix permissions: `sudo chown -R 999:999 <volume-path>` (999 is MariaDB's UID)

### Image Not Found

**Symptom**: "Error: image not found: localhost/fuel-bot:latest"

**Solution**: Build the bot image first (Step 1):
```bash
podman build -t fuel-bot:latest -f Dockerfile .
```

### Pod Already Exists

**Symptom**: "Error: pod fuel-bot-pod already exists"

**Solution**: Use the `--replace` flag or manually remove the existing pod:
```bash
podman pod stop fuel-bot-pod && podman pod rm fuel-bot-pod
podman play kube pod.yaml
```

### Health Check Failures

**Symptom**: Container marked as "unhealthy"

**Solutions**:

1. Check container logs for errors
2. Verify health check command works manually:
   ```bash
   # Database health check
   podman exec fuel-bot-pod-fuel-bot-db mariadb -e "SELECT 1"
   
   # Bot health check (if configured)
   podman exec fuel-bot-pod-fuel-bot-app pgrep -f telegram-fuel-bot
   ```

3. Adjust health check timeout/retries in pod.yaml if needed

## Advanced Operations

### Viewing Container Details

Inspect the pod configuration:
```bash
podman pod inspect fuel-bot-pod
```

Inspect individual containers:
```bash
podman inspect fuel-bot-pod-fuel-bot-app
podman inspect fuel-bot-pod-fuel-bot-db
```

### Accessing the Database

Connect to the database directly:
```bash
podman exec -it fuel-bot-pod-fuel-bot-db mariadb -u fuel_bot -pfuel_bot_internal_pass fuel_expense_bot
```

### Backing Up Data

Backup the database:
```bash
podman exec fuel-bot-pod-fuel-bot-db mariadb-dump -u fuel_bot -pfuel_bot_internal_pass fuel_expense_bot > backup.sql
```

Restore from backup:
```bash
cat backup.sql | podman exec -i fuel-bot-pod-fuel-bot-db mariadb -u fuel_bot -pfuel_bot_internal_pass fuel_expense_bot
```

### Updating the Bot

To update the bot to a new version:

1. Stop the current pod:
   ```bash
   podman pod stop fuel-bot-pod && podman pod rm fuel-bot-pod
   ```

2. Pull latest code and rebuild:
   ```bash
   git pull
   podman build -t fuel-bot:latest -f Dockerfile .
   ```

3. Redeploy:
   ```bash
   export TELEGRAM_TOKEN="your_token_here"
   envsubst < pod.yaml | podman play kube --replace -
   ```

Your data persists in the volume, so no data is lost during updates.

## Production Considerations

### Resource Limits

For production deployments, consider adding resource limits to pod.yaml:

```yaml
resources:
  limits:
    memory: "256Mi"
    cpu: "500m"
  requests:
    memory: "128Mi"
    cpu: "250m"
```

### Automatic Restart

The pod is configured with `restartPolicy: Always`, so containers automatically restart on failure.

### Monitoring

Set up monitoring for:
- Container health status
- Resource usage (CPU, memory)
- Log aggregation
- Database backup schedules

### Security

- Keep the TELEGRAM_TOKEN secure (use Kubernetes secrets in production)
- Regularly update base images for security patches
- Monitor logs for suspicious activity
- Restrict access to the host running the pod

## Quick Reference

```bash
# Build
podman build -t fuel-bot:latest -f Dockerfile .

# Create volume
podman play kube pvc.yaml

# Deploy (pass TELEGRAM_TOKEN as environment variable)
export TELEGRAM_TOKEN="your_token_here"
envsubst < pod.yaml | podman play kube --replace -

# Status
podman pod ps --filter name=fuel-bot-pod

# Logs
podman logs -f fuel-bot-pod-fuel-bot-app

# Stop
podman pod stop fuel-bot-pod && podman pod rm fuel-bot-pod
```

## Getting Help

If you encounter issues not covered in this guide:

1. Check the logs for error messages
2. Review the [README.md](README.md) for general bot information
3. Consult the [Podman documentation](https://docs.podman.io/)
4. Open an issue on the project repository
