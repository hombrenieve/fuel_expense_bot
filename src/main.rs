// Main entry point for the Telegram fuel expense tracking bot
// Implements task 11.2

mod bot;
mod config;
mod db;
mod services;
mod utils;

use std::sync::Arc;
use teloxide::Bot;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use bot::dispatcher::run_dispatcher;
use config::Config;
use db::pool::create_pool;
use db::repository::Repository;
use services::expense_service::ExpenseService;
use services::user_service::UserService;

/// Main application entry point
///
/// This function:
/// 1. Initializes structured logging (Requirement 7.5)
/// 2. Loads configuration from environment variables or config file (Requirements 8.1, 8.2, 8.3)
/// 3. Creates a database connection pool (Requirement 5.5)
/// 4. Initializes service layer (UserService, ExpenseService)
/// 5. Creates the Telegram bot instance
/// 6. Starts the bot dispatcher with graceful shutdown support (Requirements 9.1, 9.2, 9.3, 9.4)
///
/// # Graceful Shutdown
/// The bot handles SIGTERM and SIGINT signals gracefully:
/// - Stops accepting new commands
/// - Completes in-progress operations
/// - Closes database connections
/// - Logs shutdown confirmation
///
/// # Requirements
/// - Validates: Requirements 7.1, 7.4, 7.5, 8.1, 8.2, 8.3, 9.1, 9.2, 9.3, 9.4
#[tokio::main]
async fn main() {
    // Initialize structured logging (Requirement 7.5)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "telegram_fuel_bot=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Telegram fuel bot starting...");

    // Load configuration (Requirements 8.1, 8.2, 8.3)
    let config = match Config::load() {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            error!("Please ensure all required environment variables or config.toml are set");
            std::process::exit(1);
        }
    };

    // Create database connection pool (Requirement 5.5)
    let pool = match create_pool(&config.database).await {
        Ok(p) => {
            info!("Database connection pool created successfully");
            p
        }
        Err(e) => {
            error!("Failed to create database connection pool: {}", e);
            error!("Please ensure the database is running and accessible");
            std::process::exit(1);
        }
    };

    // Create repository instance
    let repository = Arc::new(Repository::new(pool.clone()));
    info!("Repository initialized");

    // Create service instances
    let user_service = Arc::new(UserService::new(repository.clone(), config.default_limit));
    info!(
        "UserService initialized with default limit: {}",
        config.default_limit
    );

    let expense_service = Arc::new(ExpenseService::new(repository.clone()));
    info!("ExpenseService initialized");

    // Initialize Telegram bot
    let bot = Bot::new(&config.telegram_token);
    info!("Telegram bot initialized");

    // Register bot commands with Telegram
    if let Err(e) = bot::dispatcher::set_bot_commands(&bot).await {
        error!("Failed to set bot commands: {}", e);
        // Continue anyway - commands will still work, just won't show in menu
    } else {
        info!("Bot commands registered with Telegram");
    }

    // Set up graceful shutdown handler
    // The dispatcher will handle SIGTERM and SIGINT via enable_ctrlc_handler()
    info!("Starting bot dispatcher with graceful shutdown support...");
    info!("Press Ctrl+C to stop the bot gracefully");

    // Start the bot dispatcher
    // This will block until a shutdown signal is received (Requirement 9.1)
    run_dispatcher(bot, user_service, expense_service).await;

    // Graceful shutdown sequence (Requirements 9.2, 9.3, 9.4)
    info!("Shutdown signal received, stopping bot...");

    // The dispatcher has already stopped accepting new commands (Requirement 9.1)
    // and completed in-progress operations (Requirement 9.2)

    // Close database connections (Requirement 9.3)
    pool.close().await;
    info!("Database connections closed");

    // Log shutdown confirmation (Requirement 9.4)
    info!("Telegram fuel bot shutdown complete");
}
