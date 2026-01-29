// Bot command handlers
// Implements task 10.1

use rust_decimal::Decimal;
use std::sync::Arc;
use teloxide::{prelude::Requester, types::Message, Bot};

use crate::services::{
    expense_service::{AddExpenseResult, ExpenseService},
    user_service::{RegistrationResult, UserService},
};
use crate::utils::error::{BotError, Result};

/// Handle /start command
///
/// Extracts the username and chat_id from the message, then calls user_service.register_user.
/// Sends a welcome message for new users or an acknowledgment for existing users.
///
/// # Requirements
/// - Validates: Requirements 1.1, 1.2
pub async fn handle_start(bot: Bot, msg: Message, user_service: Arc<UserService>) -> Result<()> {
    let chat_id = msg.chat.id.0;

    // Extract username from the message
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .ok_or_else(|| BotError::InvalidInput("No username found".to_string()))?
        .clone();

    // Register the user
    match user_service.register_user(username.clone(), chat_id).await {
        Ok(RegistrationResult::NewUser) => {
            let response = format!(
                "Welcome, {}! 🚗\n\n\
                You've been successfully registered.\n\
                Your default monthly fuel limit is €210.00.\n\n\
                Send me a number to record a fuel expense.\n\
                Use /check to see your monthly summary.\n\
                Use /config limit <amount> to change your monthly limit.",
                username
            );
            bot.send_message(msg.chat.id, response).await?;
        }
        Ok(RegistrationResult::AlreadyRegistered) => {
            let response = format!(
                "Welcome back, {}! 👋\n\n\
                You're already registered.\n\
                Send me a number to record a fuel expense or use /check to see your summary.",
                username
            );
            bot.send_message(msg.chat.id, response).await?;
        }
        Err(e) => {
            let error_msg = format_error_message(&e);
            bot.send_message(msg.chat.id, error_msg).await?;
        }
    }

    Ok(())
}

/// Handle /check command
///
/// Extracts the username from the message, calls expense_service.get_monthly_summary,
/// and formats a response showing the total spent, limit, and remaining budget.
///
/// # Requirements
/// - Validates: Requirements 3.1, 3.2, 3.3, 3.4
pub async fn handle_check(
    bot: Bot,
    msg: Message,
    expense_service: Arc<ExpenseService>,
) -> Result<()> {
    // Extract username from the message
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .ok_or_else(|| BotError::InvalidInput("No username found".to_string()))?;

    // Get the monthly summary
    match expense_service.get_monthly_summary(username).await {
        Ok(summary) => {
            let response = format!(
                "📊 Monthly Summary\n\n\
                💰 Total Spent: €{:.2}\n\
                🎯 Monthly Limit: €{:.2}\n\
                ✅ Remaining: €{:.2}",
                summary.total_spent, summary.limit, summary.remaining
            );
            bot.send_message(msg.chat.id, response).await?;
        }
        Err(e) => {
            let error_msg = format_error_message(&e);
            bot.send_message(msg.chat.id, error_msg).await?;
        }
    }

    Ok(())
}

/// Handle /config command
///
/// Parses the command arguments to extract the new limit value, validates it,
/// and calls user_service.update_limit to update the user's monthly spending limit.
///
/// Expected format: /config limit <amount>
///
/// # Requirements
/// - Validates: Requirements 4.1, 4.2, 4.3
pub async fn handle_config(
    bot: Bot,
    msg: Message,
    user_service: Arc<UserService>,
    args: Vec<String>,
) -> Result<()> {
    // Extract username from the message
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .ok_or_else(|| BotError::InvalidInput("No username found".to_string()))?;

    // Parse command arguments
    // Expected format: /config limit <amount>
    if args.len() < 2 {
        let response = "Usage: /config limit <amount>\n\nExample: /config limit 250.00";
        bot.send_message(msg.chat.id, response).await?;
        return Ok(());
    }

    if args[0].to_lowercase() != "limit" {
        let response = "Usage: /config limit <amount>\n\nExample: /config limit 250.00";
        bot.send_message(msg.chat.id, response).await?;
        return Ok(());
    }

    // Parse the amount
    let amount_str = &args[1];
    let new_limit = match amount_str.parse::<Decimal>() {
        Ok(amount) => amount,
        Err(_) => {
            let response = format!(
                "❌ Invalid amount: '{}'\n\n\
                Please enter a valid positive number.\n\
                Example: /config limit 250.00",
                amount_str
            );
            bot.send_message(msg.chat.id, response).await?;
            return Ok(());
        }
    };

    // Update the limit
    match user_service.update_limit(username, new_limit).await {
        Ok(()) => {
            let response = format!(
                "✅ Monthly limit updated!\n\n\
                Your new limit is €{:.2}",
                new_limit
            );
            bot.send_message(msg.chat.id, response).await?;
        }
        Err(e) => {
            let error_msg = format_error_message(&e);
            bot.send_message(msg.chat.id, error_msg).await?;
        }
    }

    Ok(())
}

/// Handle numeric input (expense amount)
///
/// Parses the amount and calls expense_service.add_expense to record the fuel expense.
/// Formats a response showing the result (success or limit exceeded).
///
/// # Requirements
/// - Validates: Requirements 2.1, 2.3, 2.4
pub async fn handle_numeric_input(
    bot: Bot,
    msg: Message,
    expense_service: Arc<ExpenseService>,
    amount: Decimal,
) -> Result<()> {
    // Extract username from the message
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .ok_or_else(|| BotError::InvalidInput("No username found".to_string()))?;

    // Add the expense
    match expense_service.add_expense(username, amount).await {
        Ok(AddExpenseResult::Success {
            new_total,
            remaining,
        }) => {
            let response = format!(
                "✅ Expense recorded: €{:.2}\n\n\
                📊 Monthly total: €{:.2}\n\
                💰 Remaining budget: €{:.2}",
                amount, new_total, remaining
            );
            bot.send_message(msg.chat.id, response).await?;
        }
        Ok(AddExpenseResult::LimitExceeded {
            current,
            attempted,
            limit,
        }) => {
            let response = format!(
                "❌ Expense rejected!\n\n\
                This expense of €{:.2} would exceed your monthly limit.\n\n\
                📊 Current total: €{:.2}\n\
                🎯 Monthly limit: €{:.2}\n\
                ✅ Remaining: €{:.2}\n\n\
                You can increase your limit with /config limit <amount>",
                attempted,
                current,
                limit,
                limit - current
            );
            bot.send_message(msg.chat.id, response).await?;
        }
        Err(e) => {
            let error_msg = format_error_message(&e);
            bot.send_message(msg.chat.id, error_msg).await?;
        }
    }

    Ok(())
}

/// Format error messages in a user-friendly way
///
/// This function converts internal error types into user-friendly messages
/// that don't expose implementation details or technical jargon.
///
/// # Requirements
/// - Validates: Requirement 7.3
fn format_error_message(error: &BotError) -> String {
    match error {
        BotError::Database(_) => {
            "⚠️ Unable to process your request right now. Please try again in a moment.".to_string()
        }
        BotError::Config(msg) => {
            format!("⚠️ Configuration error: {}", msg)
        }
        BotError::InvalidInput(msg) => {
            format!("❌ Invalid input: {}", msg)
        }
        BotError::UserNotFound(_) => {
            "❌ You need to register first. Please use /start to register.".to_string()
        }
        BotError::Telegram(_) => "⚠️ Unable to send message. Please try again.".to_string(),
        BotError::Parse(msg) => {
            format!("❌ Parse error: {}", msg)
        }
    }
}

#[cfg(test)]
#[path = "handlers_test.rs"]
mod handlers_test;
