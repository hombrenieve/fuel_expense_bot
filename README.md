# Fuel Expense Bot

A Telegram bot to track monthly fuel expenses and enforce spending limits. Built with Rust for reliability, type safety, and performance.

## Documentation

- **[Deployment Guide](DEPLOYMENT.md)** - Complete containerized deployment instructions and troubleshooting

## Features

- 🚗 **Track fuel expenses** - Simply send a number to record a fuel purchase
- 📊 **Monthly summaries** - Check your spending with the `/check` command
- 🎯 **Spending limits** - Set and enforce monthly budget limits
- 💰 **Budget tracking** - See remaining budget in real-time
- 🔒 **Data integrity** - Atomic transactions ensure no data loss
- ⚡ **Fast and reliable** - Built with Rust and async I/O

## Getting Started

Choose your deployment method:

1. **[Containerized Deployment](#containerized-deployment)** (Recommended) - Production-ready setup with Podman, requires only a bot token
2. **[Manual Setup](#manual-setup)** - For custom configurations or development

---

## Containerized Deployment

**Recommended for production use.** Deploy with Podman for an isolated, production-ready environment.

### Prerequisites

- Podman installed ([installation guide](https://podman.io/getting-started/installation))
- `envsubst` utility (usually pre-installed on Linux/macOS)
- Telegram bot token from [@BotFather](https://t.me/botfather)

### Quick Start

```bash
# 1. Build the bot container image
./scripts/build-image.sh

# 2. Create persistent volume for database
podman play kube pvc.yaml

# 3. Deploy the pod with your bot token
./scripts/start-pod.sh
```

### Helper Scripts

The project includes convenient scripts for common operations:

- **`./scripts/build-image.sh`** - Build the Docker image
- **`./scripts/start-pod.sh`** - Start the pod (requires TELEGRAM_TOKEN in .env)
- **`./scripts/restart-pod.sh`** - Restart the pod (useful after config changes)
- **`./scripts/rebuild-and-restart.sh`** - Build image and restart pod in one command

**Typical workflow after code changes:**
```bash
./scripts/rebuild-and-restart.sh
```

### Services Included

The pod includes three containers:
- **Telegram Bot** - The main application
- **MariaDB** - Database (internal to pod only, not exposed)
- **Adminer** - Web-based database management UI at http://localhost:8080

**Adminer Access:**
- URL: http://localhost:8080
- Server: 127.0.0.1
- Username: fuel_bot
- Password: fuel_bot_internal_pass
- Database: fuel_expense_bot

### Why Containerized?

- **Minimal configuration**: Only requires `TELEGRAM_TOKEN`
- **Isolated environment**: Bot and database run in a secure pod
- **Easy updates**: Rebuild and redeploy without affecting data
- **Production-ready**: Health checks, graceful shutdown, automatic restart

### Management

```bash
# Check status
podman pod ps --filter name=fuel-bot-pod
podman ps --filter pod=fuel-bot-pod

# View logs
podman logs -f fuel-bot-pod-fuel-bot-app

# Restart the pod
./scripts/restart-pod.sh

# Rebuild and restart after code changes
./scripts/rebuild-and-restart.sh

# Stop the pod
podman pod stop fuel-bot-pod && podman pod rm fuel-bot-pod

# Access database UI
# Open http://localhost:8080 in your browser
```

**For detailed instructions, troubleshooting, and advanced operations, see the [Deployment Guide](DEPLOYMENT.md).**

---

## Manual Setup

For development or custom configurations where you want to run the bot directly without containers.

### Prerequisites

- Rust 1.70+ ([install from rustup.rs](https://rustup.rs/))
- MariaDB 10.5+ or MySQL 8.0+
- Telegram bot token from [@BotFather](https://t.me/botfather)

### Setup Steps

1. **Set up the database:**

   **Option A: Using the setup script (Easiest)**
   ```bash
   # Run the automated setup script (works with Podman or Docker)
   ./scripts/setup-dev-db.sh
   ```
   This script automatically:
   - Detects and uses Podman or Docker
   - Creates and starts a MariaDB container
   - Initializes the database schema
   - Displays connection details for your `.env` file

   **Option B: Manual Podman setup**
   ```bash
   # Start MariaDB container
   podman run -d \
     --name fuel-bot-mariadb \
     -e MARIADB_ROOT_PASSWORD=rootpass \
     -e MARIADB_DATABASE=fuel_expense_bot \
     -e MARIADB_USER=fuel_bot \
     -e MARIADB_PASSWORD=fuel_bot_pass \
     -p 3306:3306 \
     -v fuel-bot-db:/var/lib/mysql \
     docker.io/library/mariadb:11.2
   
   # Wait a few seconds for MariaDB to start, then initialize schema
   sleep 10
   podman exec -i fuel-bot-mariadb mariadb -u fuel_bot -pfuel_bot_pass fuel_expense_bot < scripts/initdb.sql
   ```

   **Option C: Using local MariaDB/MySQL**
   ```bash
   mysql -u root -p << 'EOF'
   CREATE DATABASE fuel_expense_bot;
   CREATE USER 'fuel_bot'@'localhost' IDENTIFIED BY 'your_secure_password';
   GRANT ALL PRIVILEGES ON fuel_expense_bot.* TO 'fuel_bot'@'localhost';
   FLUSH PRIVILEGES;
   EOF
   
   mysql -u fuel_bot -p fuel_expense_bot < scripts/initdb.sql
   ```

2. **Configure environment variables:**
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials and bot token
   # (If you used the setup script, it displays the values to use)
   ```

3. **Build and run:**
   ```bash
   cargo build --release
   cargo run --release
   ```

### Environment Variables

Configuration is done through environment variables (loaded from `.env` file).

**Required:**
- `TELEGRAM_TOKEN` - Bot token from [@BotFather](https://t.me/botfather)
- `DB_HOST`, `DB_PORT`, `DB_USERNAME`, `DB_PASSWORD`, `DB_DATABASE` - Database connection settings

**Optional:**
- `DB_MAX_CONNECTIONS` - Connection pool size (default: 5)
- `DEFAULT_LIMIT` - Default monthly spending limit (default: 210.00)
- `RUST_LOG` - Logging level (default: info)

**Example `.env` file:**
```env
TELEGRAM_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz
DB_HOST=localhost
DB_PORT=3306
DB_USERNAME=fuel_bot
DB_PASSWORD=secure_password_here
DB_DATABASE=fuel_expense_bot
```

---

## Using the Bot

Once the bot is running, interact with it on Telegram:

### Commands

- **`/start`** - Register with the bot (required before recording expenses)
- **`/check`** - View your monthly spending summary
- **`/config limit <amount>`** - Set your monthly spending limit
  - Example: `/config limit 250.00`

### Recording Expenses

Simply send a number to record a fuel expense:
- `45.50` - Records €45.50 expense for today
- `100` - Records €100.00 expense for today

**Note:** If you record multiple expenses on the same day, they will be added together automatically.

### Example Conversation

```
You: /start
Bot: Welcome, alice! 🚗
     You've been successfully registered.
     Your default monthly fuel limit is €210.00.

You: 45.50
Bot: ✅ Expense recorded: €45.50
     📊 Monthly total: €45.50
     💰 Remaining budget: €164.50

You: /check
Bot: 📊 Monthly Summary
     💰 Total Spent: €45.50
     🎯 Monthly Limit: €210.00
     ✅ Remaining: €164.50

You: /config limit 300.00
Bot: ✅ Monthly limit updated!
     Your new limit is €300.00
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run property-based tests with more iterations
PROPTEST_CASES=1000 cargo test

# Run specific test module
cargo test expense_service

# Check code quality
cargo clippy

# Format code
cargo fmt
```

### Project Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── bot/
│   ├── handlers.rs      # Command handlers
│   └── dispatcher.rs    # Command routing
├── services/
│   ├── expense_service.rs  # Expense business logic
│   └── user_service.rs     # User management
├── db/
│   ├── repository.rs    # Database operations
│   ├── models.rs        # Data models
│   └── pool.rs          # Connection pooling
└── utils/
    ├── date.rs          # Date utilities
    └── error.rs         # Error types
```

## Troubleshooting

**For containerized deployments:** See the [Deployment Guide](DEPLOYMENT.md) troubleshooting section.

**For manual setup:**

### Common Issues

**Bot won't start:**
- Verify all environment variables are set in `.env`
- Check `TELEGRAM_TOKEN` is valid (get a new one from [@BotFather](https://t.me/botfather))
- Verify database is running: `mysql -u your_user -p -e "SELECT 1;"`
- Check database credentials in `.env` are correct

**Bot doesn't respond:**
- Send `/start` command first (required for registration)
- Check logs: `RUST_LOG=telegram_fuel_bot=debug cargo run`
- Verify bot is not stopped in [@BotFather](https://t.me/botfather)

**Expenses rejected:**
- Check spending: `/check`
- Increase limit: `/config limit 300.00`

### Logging

Enable detailed logging for debugging:
```bash
RUST_LOG=telegram_fuel_bot=debug cargo run
```

Log levels: `error`, `warn`, `info` (default), `debug`, `trace`

## Graceful Shutdown

The bot handles shutdown signals gracefully:
- Press `Ctrl+C` or send `SIGTERM`/`SIGINT` signal
- Bot stops accepting new commands
- In-progress operations complete
- Database connections close properly
- Shutdown confirmation logged

This ensures no data loss during shutdown.

## Database Architecture

![ER Model for DB](http://www.plantuml.com/plantuml/proxy?cache=no&src=https://raw.githubusercontent.com/hombrenieve/fuel_expense_bot/main/diagrams/febER.puml)

The database consists of two tables:

- **`config`** - Stores user configuration (username, chat ID, monthly limit)
- **`counts`** - Stores fuel expense transactions (date, username, amount)

Key features:
- Each user can have one expense per date
- Multiple expenses on the same date are automatically summed
- Monthly totals are calculated from all expenses in the current calendar month
- All operations use database transactions for data integrity



## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Run code quality checks: `cargo clippy` and `cargo fmt`
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

You are free to use, modify, and distribute this software for any purpose, including commercial use, with no restrictions.
