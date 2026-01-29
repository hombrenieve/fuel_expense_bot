# Fuel Expense Bot

A Telegram bot to track monthly fuel expenses and enforce spending limits. Built with Rust for reliability, type safety, and performance.

## Features

- 🚗 **Track fuel expenses** - Simply send a number to record a fuel purchase
- 📊 **Monthly summaries** - Check your spending with the `/check` command
- 🎯 **Spending limits** - Set and enforce monthly budget limits
- 💰 **Budget tracking** - See remaining budget in real-time
- 🔒 **Data integrity** - Atomic transactions ensure no data loss
- ⚡ **Fast and reliable** - Built with Rust and async I/O

## Getting Started

### Prerequisites

1. **Rust** (1.70 or later): Install from [https://rustup.rs/](https://rustup.rs/)
2. **MariaDB or MySQL** database (10.5+ or 8.0+)
3. **Telegram bot token** from [@BotFather](https://t.me/botfather)

### Setup

1. **Create the database schema:**
   ```bash
   mysql -u your_user -p your_database < scripts/initdb.sql
   ```

2. **Configure environment variables:**
   
   Copy the example file:
   ```bash
   cp .env.example .env
   ```

   Edit `.env` with your configuration (see [Environment Variables](#environment-variables) section below).

3. **Build and run the bot:**
   ```bash
   cargo build --release
   cargo run --release
   ```

   For development with debug logging:
   ```bash
   RUST_LOG=telegram_fuel_bot=debug cargo run
   ```

### Environment Variables

All configuration is done through environment variables. Create a `.env` file in the project root with the following variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `TELEGRAM_TOKEN` | **Yes** | - | Your Telegram bot token from [@BotFather](https://t.me/botfather) |
| `DB_HOST` | **Yes** | - | Database host (e.g., `localhost` or `127.0.0.1`) |
| `DB_PORT` | No | `3306` | Database port |
| `DB_USERNAME` | **Yes** | - | Database username |
| `DB_PASSWORD` | **Yes** | - | Database password |
| `DB_DATABASE` | **Yes** | - | Database name |
| `DB_MAX_CONNECTIONS` | No | `5` | Maximum number of database connections in the pool |
| `DEFAULT_LIMIT` | No | `210.00` | Default monthly spending limit for new users (in euros) |
| `RUST_LOG` | No | `info` | Logging level (`error`, `warn`, `info`, `debug`, `trace`) |

**Example `.env` file:**
```env
TELEGRAM_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz
DB_HOST=localhost
DB_PORT=3306
DB_USERNAME=fuel_bot
DB_PASSWORD=secure_password_here
DB_DATABASE=fuel_expense_bot
DB_MAX_CONNECTIONS=5
DEFAULT_LIMIT=210.00
RUST_LOG=telegram_fuel_bot=info
```

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

### Bot won't start

**Problem:** Bot fails to start with configuration error

**Solution:** 
- Verify all required environment variables are set in `.env`
- Check that `TELEGRAM_TOKEN` is valid (get a new one from [@BotFather](https://t.me/botfather) if needed)
- Ensure `.env` file is in the same directory where you run `cargo run`

**Problem:** Database connection error

**Solution:**
- Verify database is running: `mysql -u your_user -p -e "SELECT 1;"`
- Check `DB_HOST`, `DB_PORT`, `DB_USERNAME`, `DB_PASSWORD`, and `DB_DATABASE` are correct
- Ensure database user has proper permissions: `GRANT ALL ON fuel_expense_bot.* TO 'fuel_bot'@'localhost';`
- Verify tables exist: `mysql -u your_user -p your_database -e "SHOW TABLES;"`

### Bot doesn't respond to commands

**Problem:** Bot is running but doesn't respond to messages

**Solution:**
- Check bot logs for errors: `RUST_LOG=telegram_fuel_bot=debug cargo run`
- Verify you've sent `/start` command first (required for registration)
- Ensure your Telegram username is set (Settings → Edit Profile → Username)
- Check bot token is correct and bot is not stopped in [@BotFather](https://t.me/botfather)

### Expenses are rejected

**Problem:** "Expense rejected! This expense would exceed your monthly limit"

**Solution:**
- Check your current spending: `/check`
- Increase your limit if needed: `/config limit 300.00`
- Wait until next month for budget to reset

**Problem:** "You need to register first"

**Solution:**
- Send `/start` command to register with the bot

### Database issues

**Problem:** "Unable to process your request right now"

**Solution:**
- Check database is running and accessible
- Verify database connection settings in `.env`
- Check database logs for errors
- Ensure tables were created: `mysql -u your_user -p your_database < scripts/initdb.sql`

**Problem:** Duplicate key error or constraint violation

**Solution:**
- This usually indicates a bug. Check logs with `RUST_LOG=telegram_fuel_bot=debug`
- Verify database schema matches `scripts/initdb.sql`
- Consider recreating tables (⚠️ this will delete all data):
  ```bash
  mysql -u your_user -p your_database -e "DROP TABLE IF EXISTS counts, config;"
  mysql -u your_user -p your_database < scripts/initdb.sql
  ```

### Performance issues

**Problem:** Bot is slow to respond

**Solution:**
- Increase `DB_MAX_CONNECTIONS` in `.env` (try `10` or `20`)
- Check database server performance
- Ensure database has proper indexes (they're created by `initdb.sql`)

### Logging and debugging

To enable detailed logging:
```bash
# Debug level for bot only
RUST_LOG=telegram_fuel_bot=debug cargo run

# Trace level for everything (very verbose)
RUST_LOG=trace cargo run

# Debug for bot, info for dependencies
RUST_LOG=telegram_fuel_bot=debug,info cargo run
```

Log levels (from least to most verbose):
- `error` - Only errors
- `warn` - Warnings and errors
- `info` - General information (default)
- `debug` - Detailed debugging information
- `trace` - Very detailed trace information

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
