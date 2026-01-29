// Database repository
// Will be implemented in tasks 4.2, 4.3, 5.2, 5.3

use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sqlx::{MySql, MySqlPool, Transaction};

use super::models::{Expense, ExpenseAddResult, UserConfig};
use crate::utils::error::Result;

/// Repository trait for database operations
///
/// This trait defines the interface for all database operations required by the bot.
/// It is designed to be implemented by both the real database repository (using sqlx)
/// and a mock repository for testing purposes.
///
/// # Requirements
/// - Validates: Requirements 1.1, 2.1, 3.1, 4.1, 5.1, 5.2
#[async_trait]
pub trait RepositoryTrait: Send + Sync {
    /// Create a new user with the given username, chat ID, and default monthly limit
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `chat_id` - The Telegram chat ID
    /// * `default_limit` - The default monthly spending limit
    ///
    /// # Returns
    /// * `Ok(())` if the user was created successfully
    /// * `Err(BotError::Database)` if the user already exists or database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirement 1.1
    async fn create_user(&self, username: &str, chat_id: i64, default_limit: Decimal)
        -> Result<()>;

    /// Get user configuration by username
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(Some(UserConfig))` if the user exists
    /// * `Ok(None)` if the user does not exist
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirement 1.1
    async fn get_user_config(&self, username: &str) -> Result<Option<UserConfig>>;

    /// Update a user's monthly spending limit
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `new_limit` - The new monthly spending limit
    ///
    /// # Returns
    /// * `Ok(())` if the limit was updated successfully
    /// * `Err(BotError::Database)` if the user doesn't exist or database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirement 4.1
    async fn update_user_limit(&self, username: &str, new_limit: Decimal) -> Result<()>;

    /// Get an expense for a specific user and date
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `date` - The transaction date
    ///
    /// # Returns
    /// * `Ok(Some(Expense))` if an expense exists for that date
    /// * `Ok(None)` if no expense exists for that date
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirement 2.1
    async fn get_expense_for_date(
        &self,
        username: &str,
        date: NaiveDate,
    ) -> Result<Option<Expense>>;

    /// Create a new expense record
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `date` - The transaction date
    /// * `amount` - The expense amount
    ///
    /// # Returns
    /// * `Ok(id)` - The ID of the newly created expense
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirement 2.1
    async fn create_expense(&self, username: &str, date: NaiveDate, amount: Decimal)
        -> Result<i64>;

    /// Update an existing expense with a new amount
    ///
    /// # Arguments
    /// * `id` - The expense ID
    /// * `new_amount` - The new expense amount
    ///
    /// # Returns
    /// * `Ok(())` if the expense was updated successfully
    /// * `Err(BotError::Database)` if the expense doesn't exist or database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirement 2.1
    async fn update_expense(&self, id: i64, new_amount: Decimal) -> Result<()>;

    /// Get the total expenses for a user in a specific month
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `year` - The year (e.g., 2024)
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Ok(total)` - The sum of all expenses in that month
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirement 3.1
    async fn get_monthly_total(&self, username: &str, year: i32, month: u32) -> Result<Decimal>;

    /// Add an expense with atomic limit checking within a transaction
    ///
    /// This method performs the following operations atomically:
    /// 1. Check if an expense exists for the given date
    /// 2. Calculate what the new monthly total would be
    /// 3. Compare against the user's limit
    /// 4. If within limit: create or update the expense
    /// 5. If exceeds limit: return LimitExceeded without modifying data
    ///
    /// # Arguments
    /// * `tx` - A mutable reference to an active database transaction
    /// * `username` - The Telegram username
    /// * `date` - The transaction date
    /// * `amount` - The expense amount to add
    /// * `limit` - The user's monthly spending limit
    ///
    /// # Returns
    /// * `Ok(ExpenseAddResult::Created(id))` if a new expense was created
    /// * `Ok(ExpenseAddResult::Updated(id))` if an existing expense was updated
    /// * `Ok(ExpenseAddResult::LimitExceeded{...})` if the expense would exceed the limit
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 5.1, 5.2 (transaction support and atomicity)
    async fn add_expense_with_limit_check<'a>(
        &self,
        tx: &mut Transaction<'a, MySql>,
        username: &str,
        date: NaiveDate,
        amount: Decimal,
        limit: Decimal,
    ) -> Result<ExpenseAddResult>;
}

/// Real database repository implementation
pub struct Repository {
    pool: MySqlPool,
}

impl Repository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RepositoryTrait for Repository {
    async fn create_user(
        &self,
        username: &str,
        chat_id: i64,
        default_limit: Decimal,
    ) -> Result<()> {
        sqlx::query("INSERT INTO config (username, chatId, payLimit) VALUES (?, ?, ?)")
            .bind(username)
            .bind(chat_id)
            .bind(default_limit)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_user_config(&self, username: &str) -> Result<Option<UserConfig>> {
        let user = sqlx::query_as::<_, UserConfig>(
            "SELECT username, chatId, payLimit FROM config WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user_limit(&self, username: &str, new_limit: Decimal) -> Result<()> {
        let result = sqlx::query("UPDATE config SET payLimit = ? WHERE username = ?")
            .bind(new_limit)
            .bind(username)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(crate::utils::error::BotError::UserNotFound(
                username.to_string(),
            ));
        }

        Ok(())
    }

    async fn get_expense_for_date(
        &self,
        username: &str,
        date: NaiveDate,
    ) -> Result<Option<Expense>> {
        let expense = sqlx::query_as::<_, Expense>(
            "SELECT id, txDate, username, quantity FROM counts WHERE username = ? AND txDate = ?",
        )
        .bind(username)
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        Ok(expense)
    }

    async fn create_expense(
        &self,
        username: &str,
        date: NaiveDate,
        amount: Decimal,
    ) -> Result<i64> {
        let result =
            sqlx::query("INSERT INTO counts (txDate, username, quantity) VALUES (?, ?, ?)")
                .bind(date)
                .bind(username)
                .bind(amount)
                .execute(&self.pool)
                .await?;

        Ok(result.last_insert_id() as i64)
    }

    async fn update_expense(&self, id: i64, new_amount: Decimal) -> Result<()> {
        let result = sqlx::query("UPDATE counts SET quantity = ? WHERE id = ?")
            .bind(new_amount)
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(crate::utils::error::BotError::Database(
                sqlx::Error::Protocol(format!("Expense with id {} not found", id)),
            ));
        }

        Ok(())
    }

    async fn get_monthly_total(&self, username: &str, year: i32, month: u32) -> Result<Decimal> {
        use crate::utils::date::get_month_bounds;

        let (start_date, end_date) = get_month_bounds(year, month);

        // Query to sum all expenses for the user within the month bounds
        let result: Option<Decimal> = sqlx::query_scalar(
            "SELECT COALESCE(SUM(quantity), 0) FROM counts WHERE username = ? AND txDate >= ? AND txDate <= ?"
        )
        .bind(username)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.unwrap_or(Decimal::ZERO))
    }

    async fn add_expense_with_limit_check<'a>(
        &self,
        tx: &mut Transaction<'a, MySql>,
        username: &str,
        date: NaiveDate,
        amount: Decimal,
        limit: Decimal,
    ) -> Result<ExpenseAddResult> {
        use crate::utils::date::get_month_bounds;

        // Get the current month's total within the transaction
        let year = date.year();
        let month = date.month();
        let (start_date, end_date) = get_month_bounds(year, month);

        let current_total: Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(quantity), 0) FROM counts WHERE username = ? AND txDate >= ? AND txDate <= ?"
        )
        .bind(username)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&mut **tx)
        .await?;

        // Check if an expense exists for this date within the transaction
        let existing_expense: Option<Expense> = sqlx::query_as::<_, Expense>(
            "SELECT id, txDate, username, quantity FROM counts WHERE username = ? AND txDate = ?",
        )
        .bind(username)
        .bind(date)
        .fetch_optional(&mut **tx)
        .await?;

        // Calculate what the new total would be
        let new_total = if let Some(ref expense) = existing_expense {
            // If updating: subtract old amount, add new amount
            current_total - expense.quantity + amount
        } else {
            // If creating: just add the new amount
            current_total + amount
        };

        // Check if the new total would exceed the limit
        if new_total > limit {
            return Ok(ExpenseAddResult::LimitExceeded {
                current: current_total,
                limit,
            });
        }

        // Within limit - proceed with create or update
        if let Some(expense) = existing_expense {
            // Update existing expense within the transaction
            sqlx::query("UPDATE counts SET quantity = ? WHERE id = ?")
                .bind(amount)
                .bind(expense.id)
                .execute(&mut **tx)
                .await?;

            Ok(ExpenseAddResult::Updated(expense.id))
        } else {
            // Create new expense within the transaction
            let result =
                sqlx::query("INSERT INTO counts (txDate, username, quantity) VALUES (?, ?, ?)")
                    .bind(date)
                    .bind(username)
                    .bind(amount)
                    .execute(&mut **tx)
                    .await?;

            Ok(ExpenseAddResult::Created(result.last_insert_id() as i64))
        }
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mock repository for testing
    ///
    /// This implementation uses in-memory HashMaps to simulate database behavior
    /// without requiring an actual database connection. It simulates database
    /// constraints such as unique usernames and unique (username, date) pairs
    /// for expenses.
    ///
    /// # Requirements
    /// - Validates: Requirements 10.1, 10.7
    pub struct MockRepository {
        users: Arc<Mutex<HashMap<String, UserConfig>>>,
        expenses: Arc<Mutex<Vec<Expense>>>,
        next_expense_id: Arc<Mutex<i64>>,
    }

    impl MockRepository {
        /// Create a new empty MockRepository
        pub fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(HashMap::new())),
                expenses: Arc::new(Mutex::new(Vec::new())),
                next_expense_id: Arc::new(Mutex::new(1)),
            }
        }
    }

    impl Default for MockRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl RepositoryTrait for MockRepository {
        async fn create_user(
            &self,
            username: &str,
            chat_id: i64,
            default_limit: Decimal,
        ) -> Result<()> {
            let mut users = self.users.lock().unwrap();

            // Simulate unique username constraint
            if users.contains_key(username) {
                // Simulate a duplicate key error from the database
                return Err(crate::utils::error::BotError::Database(
                    sqlx::Error::Protocol(format!(
                        "Duplicate entry '{}' for key 'PRIMARY'",
                        username
                    )),
                ));
            }

            users.insert(
                username.to_string(),
                UserConfig {
                    username: username.to_string(),
                    chat_id,
                    pay_limit: default_limit,
                },
            );

            Ok(())
        }

        async fn get_user_config(&self, username: &str) -> Result<Option<UserConfig>> {
            let users = self.users.lock().unwrap();
            Ok(users.get(username).cloned())
        }

        async fn update_user_limit(&self, username: &str, new_limit: Decimal) -> Result<()> {
            let mut users = self.users.lock().unwrap();

            match users.get_mut(username) {
                Some(user) => {
                    user.pay_limit = new_limit;
                    Ok(())
                }
                None => {
                    // Simulate a "no rows affected" error
                    Err(crate::utils::error::BotError::UserNotFound(
                        username.to_string(),
                    ))
                }
            }
        }

        async fn get_expense_for_date(
            &self,
            username: &str,
            date: NaiveDate,
        ) -> Result<Option<Expense>> {
            let expenses = self.expenses.lock().unwrap();
            Ok(expenses
                .iter()
                .find(|e| e.username == username && e.tx_date == date)
                .cloned())
        }

        async fn create_expense(
            &self,
            username: &str,
            date: NaiveDate,
            amount: Decimal,
        ) -> Result<i64> {
            let mut expenses = self.expenses.lock().unwrap();
            let mut next_id = self.next_expense_id.lock().unwrap();

            // Check for unique (username, date) constraint
            if expenses
                .iter()
                .any(|e| e.username == username && e.tx_date == date)
            {
                return Err(crate::utils::error::BotError::Database(
                    sqlx::Error::Protocol(format!(
                        "Duplicate entry '{}-{}' for key 'unique_user_date'",
                        username, date
                    )),
                ));
            }

            let id = *next_id;
            *next_id += 1;

            expenses.push(Expense {
                id,
                tx_date: date,
                username: username.to_string(),
                quantity: amount,
            });

            Ok(id)
        }

        async fn update_expense(&self, id: i64, new_amount: Decimal) -> Result<()> {
            let mut expenses = self.expenses.lock().unwrap();

            match expenses.iter_mut().find(|e| e.id == id) {
                Some(expense) => {
                    expense.quantity = new_amount;
                    Ok(())
                }
                None => Err(crate::utils::error::BotError::Database(
                    sqlx::Error::Protocol(format!("Expense with id {} not found", id)),
                )),
            }
        }

        async fn get_monthly_total(
            &self,
            username: &str,
            year: i32,
            month: u32,
        ) -> Result<Decimal> {
            let expenses = self.expenses.lock().unwrap();

            // Calculate month boundaries
            let start_date = NaiveDate::from_ymd_opt(year, month, 1).ok_or_else(|| {
                crate::utils::error::BotError::InvalidInput(format!(
                    "Invalid date: year={}, month={}",
                    year, month
                ))
            })?;

            // Get the last day of the month
            let end_date = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1).and_then(|d| d.pred_opt())
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1).and_then(|d| d.pred_opt())
            }
            .ok_or_else(|| {
                crate::utils::error::BotError::InvalidInput(format!(
                    "Invalid date calculation for year={}, month={}",
                    year, month
                ))
            })?;

            // Sum all expenses for this user in the date range
            let total = expenses
                .iter()
                .filter(|e| {
                    e.username == username && e.tx_date >= start_date && e.tx_date <= end_date
                })
                .map(|e| e.quantity)
                .sum();

            Ok(total)
        }

        async fn add_expense_with_limit_check<'a>(
            &self,
            _tx: &mut Transaction<'a, MySql>,
            username: &str,
            date: NaiveDate,
            amount: Decimal,
            limit: Decimal,
        ) -> Result<ExpenseAddResult> {
            // Note: In the mock, we ignore the transaction parameter since we're using
            // in-memory data structures. The real implementation will use the transaction.

            // Get the current month's total
            let year = date.year();
            let month = date.month();
            let current_total = self.get_monthly_total(username, year, month).await?;

            // Check if an expense exists for this date
            let existing_expense = self.get_expense_for_date(username, date).await?;

            // Calculate what the new total would be
            let new_total = if let Some(ref expense) = existing_expense {
                // If updating: subtract old amount, add new amount
                current_total - expense.quantity + amount
            } else {
                // If creating: just add the new amount
                current_total + amount
            };

            // Check if the new total would exceed the limit
            if new_total > limit {
                return Ok(ExpenseAddResult::LimitExceeded {
                    current: current_total,
                    limit,
                });
            }

            // Within limit - proceed with create or update
            if let Some(expense) = existing_expense {
                // Update existing expense
                self.update_expense(expense.id, amount).await?;
                Ok(ExpenseAddResult::Updated(expense.id))
            } else {
                // Create new expense
                let id = self.create_expense(username, date, amount).await?;
                Ok(ExpenseAddResult::Created(id))
            }
        }
    }
}

#[cfg(test)]
#[path = "repository_test.rs"]
mod repository_test;
