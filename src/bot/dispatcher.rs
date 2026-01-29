// Bot dispatcher and command routing
// Implements task 11.1

use rust_decimal::Decimal;
use std::sync::Arc;
use teloxide::{
    dispatching::UpdateFilterExt,
    dptree,
    prelude::*,
    types::{Message, Update},
    utils::command::BotCommands,
    Bot,
};
use tracing::{error, info};

use crate::bot::handlers::{handle_check, handle_config, handle_numeric_input, handle_start};
use crate::services::{expense_service::ExpenseService, user_service::UserService};

/// Bot commands enum for teloxide command parsing
#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Fuel expense tracking commands:"
)]
enum Command {
    #[command(description = "Register with the bot")]
    Start,
    #[command(description = "Check your monthly spending summary")]
    Check,
    #[command(description = "Configure your monthly limit (usage: /config limit <amount>)")]
    Config(String),
}

/// Set up and run the bot dispatcher
///
/// This function sets up the teloxide dispatcher with command routing:
/// - /start -> handle_start
/// - /check -> handle_check
/// - /config -> handle_config
/// - Numeric text messages -> handle_numeric_input
///
/// All incoming commands are logged for audit purposes.
///
/// # Requirements
/// - Validates: Requirement 7.4
pub async fn run_dispatcher(
    bot: Bot,
    user_service: Arc<UserService>,
    expense_service: Arc<ExpenseService>,
) {
    info!("Starting bot dispatcher...");

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(
            dptree::entry()
                .filter_map(|msg: Message| {
                    // Try to parse the message text as a decimal number
                    msg.text()
                        .and_then(|text| text.parse::<Decimal>().ok())
                        .map(|amount| (msg, amount))
                })
                .endpoint(numeric_handler),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![user_service, expense_service])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    info!("Bot dispatcher stopped");
}

/// Handler for bot commands
async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    user_service: Arc<UserService>,
    expense_service: Arc<ExpenseService>,
) -> ResponseResult<()> {
    // Log incoming command
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("unknown");

    let chat_id = msg.chat.id.0;

    match &cmd {
        Command::Start => {
            info!(
                "Received /start command from user: {}, chat_id: {}",
                username, chat_id
            );
            if let Err(e) = handle_start(bot, msg, user_service).await {
                error!("Error handling /start command: {:?}", e);
            }
        }
        Command::Check => {
            info!(
                "Received /check command from user: {}, chat_id: {}",
                username, chat_id
            );
            if let Err(e) = handle_check(bot, msg, expense_service).await {
                error!("Error handling /check command: {:?}", e);
            }
        }
        Command::Config(args_str) => {
            info!(
                "Received /config command from user: {}, chat_id: {}, args: {}",
                username, chat_id, args_str
            );
            // Parse the arguments string into a vector
            let args: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
            if let Err(e) = handle_config(bot, msg, user_service, args).await {
                error!("Error handling /config command: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Handler for numeric text messages (expense amounts)
async fn numeric_handler(
    bot: Bot,
    msg: Message,
    amount: Decimal,
    expense_service: Arc<ExpenseService>,
) -> ResponseResult<()> {
    // Log incoming numeric input
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("unknown");

    let chat_id = msg.chat.id.0;

    info!(
        "Received numeric input from user: {}, chat_id: {}, amount: {}",
        username, chat_id, amount
    );

    if let Err(e) = handle_numeric_input(bot, msg, expense_service, amount).await {
        error!("Error handling numeric input: {:?}", e);
    }

    Ok(())
}
