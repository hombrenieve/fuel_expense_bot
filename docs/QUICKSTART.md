# Quick Start Guide - Fuel Expense Bot

This guide will get you up and running in 5 minutes using Podman/Docker and a local database.

## Prerequisites

- Rust installed ([rustup.rs](https://rustup.rs/))
- Podman or Docker installed
- A Telegram account

## Step-by-Step Instructions

### 1. Get Your Bot Token

1. Open Telegram and search for `@BotFather`
2. Send `/newbot` and follow the instructions
3. Copy your bot token (looks like `1234567890:ABCdefGHIjklMNOpqrsTUVwxyz`)

### 2. Setup Database

**Option A: Automated (Recommended)**

```bash
./scripts/setup-dev-db.sh
```

This script automatically sets up everything. Skip to step 4 after running it.

**Option B: Manual Setup**

```bash
# Create persistent volume
podman volume create fuel_bot_data

# Start MariaDB
podman run -d \
  --name fuel_bot_db \
  -e MYSQL_ROOT_PASSWORD=rootpass \
  -e MYSQL_DATABASE=fuel_expense_bot \
  -e MYSQL_USER=fuel_bot \
  -e MYSQL_PASSWORD=botpass \
  -p 3306:3306 \
  -v fuel_bot_data:/var/lib/mysql \
  docker.io/library/mariadb:latest

# Wait for database to start
sleep 5
```

**Using Docker?** Replace `podman` with `docker` in all commands.

### 3. Initialize Database Schema (Manual Setup Only)

```bash
# Load the schema
podman exec -i fuel_bot_db mysql -ufuel_bot -pbotpass fuel_expense_bot < scripts/initdb.sql

# Verify it worked
podman exec fuel_bot_db mysql -ufuel_bot -pbotpass fuel_expense_bot -e "SHOW TABLES;"
```

Expected output:
```
+-----------------------------+
| Tables_in_fuel_expense_bot  |
+-----------------------------+
| config                      |
| counts                      |
+-----------------------------+
```

### 4. Configure the Bot

Create `.env` file with your bot token:

```bash
cat > .env << 'EOF'
TELEGRAM_TOKEN=YOUR_BOT_TOKEN_HERE
DB_HOST=localhost
DB_PORT=3306
DB_USERNAME=fuel_bot
DB_PASSWORD=botpass
DB_DATABASE=fuel_expense_bot
EOF
```

**Important:** Replace `YOUR_BOT_TOKEN_HERE` with your actual token from step 1!

**Note:** Only these 6 variables are required. The bot has sensible defaults for everything else.

### 5. Build and Run

```bash
# Build (first time takes a few minutes)
cargo build --release

# Run the bot
cargo run --release
```

You should see:
```
INFO telegram_fuel_bot: Telegram fuel bot starting...
INFO telegram_fuel_bot: Configuration loaded successfully
INFO telegram_fuel_bot: Database connection pool created successfully
INFO telegram_fuel_bot: Starting bot dispatcher...
```

### 6. Test It!

1. Open Telegram and find your bot
2. Send `/start` - you should get a welcome message
3. Send `45.50` - records a €45.50 expense
4. Send `/check` - shows your monthly summary

## Common Commands

### Stop Everything
```bash
# Stop bot: Press Ctrl+C in the terminal

# Stop database
podman stop fuel_bot_db
```

### Restart Later
```bash
# Start database
podman start fuel_bot_db
sleep 3

# Run bot
cargo run --release
```

### Clean Up (Delete Everything)
```bash
# Stop and remove container
podman stop fuel_bot_db
podman rm fuel_bot_db

# Remove volume (⚠️ deletes all data!)
podman volume rm fuel_bot_data
```

### View Database Data
```bash
# Connect to database
podman exec -it fuel_bot_db mysql -ufuel_bot -pbotpass fuel_expense_bot

# Then run SQL queries:
# SELECT * FROM config;
# SELECT * FROM counts;
# exit
```

### View Bot Logs with Debug Info
```bash
RUST_LOG=telegram_fuel_bot=debug cargo run --release
```

## Troubleshooting

### "Failed to create database connection pool"
- Check database is running: `podman ps | grep fuel_bot_db`
- If not running: `podman start fuel_bot_db`
- Wait a few seconds and try again

### "Missing required configuration: TELEGRAM_TOKEN"
- Make sure `.env` file exists in project root
- Check the token is correct (no extra spaces)
- Try setting it directly: `export TELEGRAM_TOKEN=your_token_here`

### Bot doesn't respond
- Check bot is running (you should see logs in terminal)
- Make sure you sent `/start` first
- Check your Telegram username is set (Settings → Edit Profile → Username)

### Port 3306 already in use
- Another database is running on that port
- Either stop it, or use a different port:
  ```bash
  # Use port 3307 instead
  podman run -d ... -p 3307:3306 ...
  
  # Update .env
  DB_PORT=3307
  ```

## Next Steps

- Read the full [README.md](../README.md) for more features
- Configure your monthly limit: `/config limit 300.00`
- Check the [Troubleshooting section](../README.md#troubleshooting) for more help

## Quick Reference

**Bot Commands:**
- `/start` - Register with the bot
- `/check` - View monthly summary
- `/config limit <amount>` - Set monthly limit
- `<number>` - Record an expense

**Useful Podman Commands:**
- `podman ps` - List running containers
- `podman logs fuel_bot_db` - View database logs
- `podman exec -it fuel_bot_db bash` - Shell into container
- `podman volume ls` - List volumes
- `podman volume inspect fuel_bot_data` - Volume details
