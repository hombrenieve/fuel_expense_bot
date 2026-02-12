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
                📝 Commands:\n\
                • Send a number to record a fuel expense\n\
                • /check - See your monthly summary\n\
                • /list_month - List all expenses this month\n\
                • /year_summary - View yearly expense summary\n\
                • /remove_last - Remove the last expense\n\
                • /clear_month - Clear all expenses this month\n\
                • /config limit <amount> - Change your monthly limit",
                username
            );
            bot.send_message(msg.chat.id, response).await?;
        }
        Ok(RegistrationResult::AlreadyRegistered) => {
            let response = format!(
                "Welcome back, {}! 👋\n\n\
                You're already registered.\n\n\
                📝 Commands:\n\
                • Send a number to record a fuel expense\n\
                • /check - See your monthly summary\n\
                • /list_month - List all expenses this month\n\
                • /year_summary - View yearly expense summary\n\
                • /remove_last - Remove the last expense\n\
                • /clear_month - Clear all expenses this month\n\
                • /config limit <amount> - Change your monthly limit",
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

/// Handle /list_month command
///
/// Extracts the username from the message, calls expense_service.list_current_month_expenses,
/// and formats a response showing all expenses in the current month with day and amount.
///
/// # Requirements
/// - Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5
pub async fn handle_list_month(
    bot: Bot,
    msg: Message,
    expense_service: Arc<ExpenseService>,
) -> Result<()> {
    // Extract username from the message
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .ok_or_else(|| BotError::InvalidInput("No username found".to_string()))?;

    // Get the current month's expenses
    match expense_service.list_current_month_expenses(username).await {
        Ok(expenses) => {
            if expenses.is_empty() {
                // Handle empty month case
                let response = "📋 Current Month Expenses\n\nNo expenses recorded this month.";
                bot.send_message(msg.chat.id, response).await?;
            } else {
                // Format response with day and amount for each expense
                let mut response = String::from("📋 Current Month Expenses\n\n");
                for expense in expenses {
                    response.push_str(&format!("Day {}: €{:.2}\n", expense.day, expense.amount));
                }
                bot.send_message(msg.chat.id, response).await?;
            }
        }
        Err(e) => {
            let error_msg = format_error_message(&e);
            bot.send_message(msg.chat.id, error_msg).await?;
        }
    }

    Ok(())
}

/// Handle /year_summary command
///
/// Extracts the username from the message, calls expense_service.get_year_summary,
/// and formats a response showing monthly totals and grand total for the current year.
///
/// # Requirements
/// - Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5
pub async fn handle_year_summary(
    bot: Bot,
    msg: Message,
    expense_service: Arc<ExpenseService>,
) -> Result<()> {
    // Extract username from the message
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .ok_or_else(|| BotError::InvalidInput("No username found".to_string()))?;

    // Get the year summary
    match expense_service.get_year_summary(username).await {
        Ok(summary) => {
            if summary.monthly_totals.is_empty() {
                // Handle empty year case
                let response = format!(
                    "📊 Year Summary {}\n\nNo expenses recorded this year.",
                    summary.year
                );
                bot.send_message(msg.chat.id, response).await?;
            } else {
                // Format response with month names, totals, and grand total
                let mut response = format!("📊 Year Summary {}\n\n", summary.year);
                for month_total in summary.monthly_totals {
                    response.push_str(&format!(
                        "{}: €{:.2}\n",
                        month_total.month_name, month_total.total
                    ));
                }
                response.push_str(&format!("\n💰 Grand Total: €{:.2}", summary.grand_total));
                bot.send_message(msg.chat.id, response).await?;
            }
        }
        Err(e) => {
            let error_msg = format_error_message(&e);
            bot.send_message(msg.chat.id, error_msg).await?;
        }
    }

    Ok(())
}

/// Handle /clear_month command
///
/// Extracts the username from the message, calls expense_service.clear_current_month,
/// and formats a confirmation message with the count of deleted expenses.
///
/// # Requirements
/// - Validates: Requirements 3.1, 3.2, 3.3, 3.4
pub async fn handle_clear_month(
    bot: Bot,
    msg: Message,
    expense_service: Arc<ExpenseService>,
) -> Result<()> {
    // Extract username from the message
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .ok_or_else(|| BotError::InvalidInput("No username found".to_string()))?;

    // Clear current month expenses
    match expense_service.clear_current_month(username).await {
        Ok(deleted_count) => {
            if deleted_count == 0 {
                // Handle empty month case
                let response = "🗑️ Clear Month\n\nNo expenses to clear this month.";
                bot.send_message(msg.chat.id, response).await?;
            } else {
                // Format confirmation message with count
                let response = format!(
                    "✅ Month Cleared\n\n{} expense{} removed from current month.",
                    deleted_count,
                    if deleted_count == 1 { "" } else { "s" }
                );
                bot.send_message(msg.chat.id, response).await?;
            }
        }
        Err(e) => {
            let error_msg = format_error_message(&e);
            bot.send_message(msg.chat.id, error_msg).await?;
        }
    }

    Ok(())
}

/// Handle /remove_last command
///
/// Extracts the username from the message, calls expense_service.remove_last_expense,
/// and formats a confirmation message with the deleted expense details.
///
/// # Requirements
/// - Validates: Requirements 4.1, 4.2, 4.3, 4.4, 5.3
pub async fn handle_remove_last(
    bot: Bot,
    msg: Message,
    expense_service: Arc<ExpenseService>,
) -> Result<()> {
    // Extract username from the message
    let username = msg
        .from()
        .and_then(|user| user.username.as_ref())
        .ok_or_else(|| BotError::InvalidInput("No username found".to_string()))?;

    // Remove last expense
    match expense_service.remove_last_expense(username).await {
        Ok(Some(expense)) => {
            // Format confirmation message with deleted expense details
            let response = format!(
                "✅ Last Expense Removed\n\nDay {}: €{:.2}",
                expense.day, expense.amount
            );
            bot.send_message(msg.chat.id, response).await?;
        }
        Ok(None) => {
            // Handle empty month case
            let response = "🗑️ Remove Last Expense\n\nNo expenses to remove this month.";
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
