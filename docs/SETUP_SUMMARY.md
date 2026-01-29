# Setup Summary - What You Need to Know

## TL;DR - Minimal Setup

You only need **6 environment variables** to run the bot:

```env
TELEGRAM_TOKEN=your_token_from_botfather
DB_HOST=localhost
DB_PORT=3306
DB_USERNAME=fuel_bot
DB_PASSWORD=botpass
DB_DATABASE=fuel_expense_bot
```

Everything else has sensible defaults!

## Quick Commands

### Setup (One Time)
```bash
# 1. Get bot token from @BotFather on Telegram

# 2. Setup database (automated)
./scripts/setup-dev-db.sh

# 3. Create .env with your token
cat > .env << 'EOF'
TELEGRAM_TOKEN=your_token_here
DB_HOST=localhost
DB_PORT=3306
DB_USERNAME=fuel_bot
DB_PASSWORD=botpass
DB_DATABASE=fuel_expense_bot
EOF

# 4. Build and run
cargo build --release
cargo run --release
```

### Daily Use
```bash
# Start database (if not running)
podman start fuel_bot_db

# Run bot
cargo run --release

# Stop bot: Ctrl+C
```

## What Gets Defaults?

These variables are **optional** and have defaults:

| Variable | Default | What it does |
|----------|---------|--------------|
| `DB_MAX_CONNECTIONS` | `5` | Database connection pool size |
| `DEFAULT_LIMIT` | `210.00` | Default monthly limit for new users |
| `RUST_LOG` | `info` | Logging verbosity |

You don't need to set them unless you want to change the defaults.

## Files Created

- `.env` - Your configuration (you create this)
- `docs/QUICKSTART.md` - Detailed step-by-step guide
- `scripts/setup-dev-db.sh` - Automated database setup script
- Podman volume `fuel_bot_data` - Persistent database storage

## Architecture

```
Your Machine
├── Bot (Rust process)
│   └── Reads .env for config
└── Database (Podman container)
    └── Stores data in volume
```

The bot connects to the database on localhost:3306.

## Persistence

- **Database data**: Stored in Podman volume `fuel_bot_data`
- **Configuration**: Stored in `.env` file
- **Code**: Your git repository

Even if you stop/remove the container, your data is safe in the volume!

## Next Steps

1. Follow [QUICKSTART.md](QUICKSTART.md) for detailed instructions
2. Read [README.md](../README.md) for full documentation
3. Test the bot on Telegram!

## Need Help?

- **Quick start**: See [QUICKSTART.md](QUICKSTART.md)
- **Full docs**: See [README.md](../README.md)
- **Troubleshooting**: See README.md troubleshooting section
