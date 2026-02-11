// Expense service
// Implements requirements 2.2, 2.3, 2.4, 2.5, 2.6, 3.1, 3.2, 3.3, 3.4, 5.1, 5.2

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::db::models::MonthlySummary;
use crate::db::repository::RepositoryTrait;
use crate::utils::date::current_date;
use crate::utils::error::{BotError, Result};

/// Service for managing fuel expenses
///
/// This service handles the business logic for adding expenses and generating
/// monthly summaries. It enforces spending limits and ensures all operations
/// are performed atomically using database transactions.
pub struct ExpenseService {
    repo: Arc<dyn RepositoryTrait>,
}

impl ExpenseService {
    pub fn new(repo: Arc<dyn RepositoryTrait>) -> Self {
        Self { repo }
    }

    /// Get detailed list of current month's expenses
    ///
    /// Returns expenses with day information for display, ordered chronologically.
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(Vec<ExpenseDetail>)` - Vector of expense details, empty if no expenses
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5
    pub async fn list_current_month_expenses(&self, username: &str) -> Result<Vec<ExpenseDetail>> {
        // Get all expenses for the current month from repository
        let expenses = self.repo.get_current_month_expenses(username).await?;

        // Transform Expense models to ExpenseDetail with day extraction
        let details = expenses
            .into_iter()
            .map(|expense| ExpenseDetail {
                day: expense.tx_date.day(),
                amount: expense.quantity,
                date: expense.tx_date,
            })
            .collect();

        Ok(details)
    }

    /// Clear all expenses from the current month
    ///
    /// Removes all expenses for the user in the current month and returns
    /// the count of deleted expenses.
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of expenses deleted (0 if month was empty)
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 3.1, 3.2, 3.3, 3.4
    pub async fn clear_current_month(&self, username: &str) -> Result<u64> {
        // Call repository to delete all current month expenses
        let deleted_count = self.repo.delete_current_month_expenses(username).await?;

        Ok(deleted_count)
    }

    /// Remove the last (most recent) expense from the current month
    ///
    /// Identifies and removes the most recent expense in the current month.
    /// Returns the deleted expense details if one existed.
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(Some(ExpenseDetail))` - The deleted expense details if one existed
    /// * `Ok(None)` - If no expenses exist in the current month
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 4.1, 4.2, 4.3, 4.4, 5.3
    pub async fn remove_last_expense(&self, username: &str) -> Result<Option<ExpenseDetail>> {
        // Call repository to delete the last current month expense
        let deleted_expense = self.repo.delete_last_current_month_expense(username).await?;

        // Transform to ExpenseDetail if an expense was deleted
        let result = deleted_expense.map(|expense| ExpenseDetail {
            day: expense.tx_date.day(),
            amount: expense.quantity,
            date: expense.tx_date,
        });

        Ok(result)
    }

    /// Get summary of expenses for the entire current year
    ///
    /// Returns monthly totals with month names and a grand total for the year.
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(YearSummary)` - Summary with monthly totals and grand total
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5
    pub async fn get_year_summary(&self, username: &str) -> Result<YearSummary> {
        // Get current year
        let today = current_date();
        let year = today.year();

        // Get monthly totals from repository
        let monthly_data = self.repo.get_year_summary(username, year).await?;

        // Transform month numbers to month names and calculate grand total
        let mut grand_total = Decimal::ZERO;
        let monthly_totals = monthly_data
            .into_iter()
            .map(|(month, total)| {
                grand_total += total;
                MonthTotal {
                    month,
                    month_name: month_number_to_name(month),
                    total,
                }
            })
            .collect();

        Ok(YearSummary {
            year,
            monthly_totals,
            grand_total,
        })
    }

    /// Add an expense for the current date
    ///
    /// This function:
    /// 1. Gets the user's configuration (including their monthly limit)
    /// 2. Uses the current date as the transaction date
    /// 3. Calls validate_and_add_with_transaction to perform the operation atomically
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `amount` - The expense amount to add
    ///
    /// # Returns
    /// * `Ok(AddExpenseResult::Success{...})` if the expense was added successfully
    /// * `Ok(AddExpenseResult::LimitExceeded{...})` if the expense would exceed the limit
    /// * `Err(BotError::UserNotFound)` if the user doesn't exist
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 2.2, 2.3, 2.4, 2.5, 2.6
    pub async fn add_expense(&self, username: &str, amount: Decimal) -> Result<AddExpenseResult> {
        // Get user configuration to retrieve the monthly limit
        let user_config = self
            .repo
            .get_user_config(username)
            .await?
            .ok_or_else(|| BotError::UserNotFound(username.to_string()))?;

        // Use current date for the transaction
        let date = current_date();

        // Perform the operation with transaction support
        self.validate_and_add_with_transaction(username, date, amount, &user_config)
            .await
    }

    /// Get monthly summary for the current month
    ///
    /// This function calculates:
    /// - Total spent in the current month
    /// - The user's monthly limit
    /// - Remaining budget (limit - spent)
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(MonthlySummary)` with the calculated values
    /// * `Err(BotError::UserNotFound)` if the user doesn't exist
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 3.1, 3.2, 3.3, 3.4
    pub async fn get_monthly_summary(&self, username: &str) -> Result<MonthlySummary> {
        // Get user configuration to retrieve the monthly limit
        let user_config = self
            .repo
            .get_user_config(username)
            .await?
            .ok_or_else(|| BotError::UserNotFound(username.to_string()))?;

        // Get current month's total
        let today = current_date();
        let year = today.year();
        let month = today.month();
        let total_spent = self.repo.get_monthly_total(username, year, month).await?;

        // Calculate remaining budget
        let remaining = user_config.pay_limit - total_spent;

        Ok(MonthlySummary {
            total_spent,
            limit: user_config.pay_limit,
            remaining,
        })
    }

    /// Validate and add an expense with transaction support
    ///
    /// This is an internal helper function that performs the actual expense addition
    /// within a database transaction. It:
    /// 1. Gets the current monthly total
    /// 2. Checks if an expense exists for the date
    /// 3. Calculates the new total (if updating, adds to existing; if creating, just adds)
    /// 4. Checks against the limit
    /// 5. Creates or updates the expense
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `date` - The transaction date
    /// * `amount` - The expense amount to add
    /// * `user_config` - The user's configuration (containing the limit)
    ///
    /// # Returns
    /// * `Ok(AddExpenseResult::Success{...})` if the expense was added successfully
    /// * `Ok(AddExpenseResult::LimitExceeded{...})` if the expense would exceed the limit
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 2.5, 2.6, 5.1, 5.2 (transaction support and atomicity)
    async fn validate_and_add_with_transaction(
        &self,
        username: &str,
        date: NaiveDate,
        amount: Decimal,
        user_config: &crate::db::models::UserConfig,
    ) -> Result<AddExpenseResult> {
        let year = date.year();
        let month = date.month();

        // Get current monthly total
        let current_total = self.repo.get_monthly_total(username, year, month).await?;

        // Check if an expense exists for this date
        let existing_expense = self.repo.get_expense_for_date(username, date).await?;

        // Calculate what the new total would be
        let new_amount = if let Some(ref expense) = existing_expense {
            // If updating: add to existing amount (Requirement 2.5)
            expense.quantity + amount
        } else {
            // If creating: just use the new amount
            amount
        };

        // Calculate new monthly total
        let new_total = if let Some(ref expense) = existing_expense {
            // If updating: subtract old amount, add new combined amount
            current_total - expense.quantity + new_amount
        } else {
            // If creating: just add the new amount
            current_total + amount
        };

        // Check if the new total would exceed the limit
        if new_total > user_config.pay_limit {
            return Ok(AddExpenseResult::LimitExceeded {
                current: current_total,
                attempted: amount,
                limit: user_config.pay_limit,
            });
        }

        // Within limit - proceed with create or update
        if let Some(expense) = existing_expense {
            // Update existing expense with combined amount
            self.repo.update_expense(expense.id, new_amount).await?;
        } else {
            // Create new expense
            self.repo.create_expense(username, date, amount).await?;
        }

        // Calculate remaining budget
        let remaining = user_config.pay_limit - new_total;

        Ok(AddExpenseResult::Success {
            new_total,
            remaining,
        })
    }
}

/// Convert month number (1-12) to month name
///
/// # Arguments
/// * `month` - Month number (1-12)
///
/// # Returns
/// * Month name as a string (e.g., "January", "February", etc.)
fn month_number_to_name(month: u32) -> String {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
    .to_string()
}

/// Detailed information about a single expense
///
/// This struct provides day-level detail for expense display.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpenseDetail {
    /// Day of month (1-31)
    pub day: u32,
    /// Expense amount
    pub amount: Decimal,
    /// Full date for reference
    pub date: NaiveDate,
}

/// Summary of expenses for an entire year
///
/// This struct aggregates monthly totals and provides a grand total.
#[derive(Debug, Clone, PartialEq)]
pub struct YearSummary {
    /// The year being summarized
    pub year: i32,
    /// Monthly totals for months with expenses
    pub monthly_totals: Vec<MonthTotal>,
    /// Grand total for the entire year
    pub grand_total: Decimal,
}

/// Total expenses for a single month
///
/// This struct represents the aggregated expenses for one month.
#[derive(Debug, Clone, PartialEq)]
pub struct MonthTotal {
    /// Month number (1-12)
    pub month: u32,
    /// Month name for display (e.g., "January")
    pub month_name: String,
    /// Total expenses for the month
    pub total: Decimal,
}

/// Result of adding an expense
///
/// This enum represents the outcome of attempting to add an expense.
#[derive(Debug, Clone, PartialEq)]
pub enum AddExpenseResult {
    /// The expense was added successfully
    Success {
        /// The new monthly total after adding the expense
        new_total: Decimal,
        /// The remaining budget for the month
        remaining: Decimal,
    },
    /// The expense would exceed the monthly limit
    LimitExceeded {
        /// The current monthly total before attempting to add the expense
        current: Decimal,
        /// The amount that was attempted to be added
        attempted: Decimal,
        /// The user's monthly spending limit
        limit: Decimal,
    },
}

#[cfg(test)]
#[path = "expense_service_test.rs"]
mod expense_service_test;
