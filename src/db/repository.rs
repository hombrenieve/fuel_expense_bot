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

    /// Get all expenses for a user in the current month with detailed information
    ///
    /// Returns expenses ordered chronologically by date (ascending), with ID descending
    /// as a tiebreaker for same-day expenses.
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(Vec<Expense>)` - Vector of expenses in the current month, ordered chronologically
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 1.1, 1.5
    async fn get_current_month_expenses(&self, username: &str) -> Result<Vec<Expense>>;

    /// Delete all expenses for a user in the current month
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(u64)` - The number of expenses deleted
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 3.1, 3.2
    async fn delete_current_month_expenses(&self, username: &str) -> Result<u64>;

    /// Delete the most recent expense for a user in the current month
    ///
    /// Identifies the most recent expense by date (descending), with ID as tiebreaker
    /// for same-day expenses.
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(Some(Expense))` - The deleted expense if one existed
    /// * `Ok(None)` - If no expenses exist in the current month
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 4.1, 4.2, 4.4
    async fn delete_last_current_month_expense(&self, username: &str) -> Result<Option<Expense>>;

    /// Get monthly totals for the entire current year
    ///
    /// Returns a vector of (month, total) tuples for months with expenses.
    /// Months with no expenses are omitted from results.
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `year` - The year to summarize
    ///
    /// # Returns
    /// * `Ok(Vec<(u32, Decimal)>)` - Vector of (month_number, total) tuples ordered by month
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 2.1, 2.4
    async fn get_year_summary(&self, username: &str, year: i32) -> Result<Vec<(u32, Decimal)>>;

    /// Get all active chat IDs for startup notifications
    ///
    /// Returns a list of unique chat IDs from the config table.
    ///
    /// # Returns
    /// * `Ok(Vec<i64>)` - Vector of unique chat IDs
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirement 6.1
    async fn get_all_chat_ids(&self) -> Result<Vec<i64>>;
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

    async fn get_current_month_expenses(&self, username: &str) -> Result<Vec<Expense>> {
        use chrono::Local;

        let now = Local::now().date_naive();
        let year = now.year();
        let month = now.month();

        let expenses = sqlx::query_as::<_, Expense>(
            "SELECT id, txDate, username, quantity FROM counts 
             WHERE username = ? AND YEAR(txDate) = ? AND MONTH(txDate) = ? 
             ORDER BY txDate ASC, id DESC"
        )
        .bind(username)
        .bind(year)
        .bind(month)
        .fetch_all(&self.pool)
        .await?;

        Ok(expenses)
    }

    async fn delete_current_month_expenses(&self, username: &str) -> Result<u64> {
        use chrono::Local;

        let now = Local::now().date_naive();
        let year = now.year();
        let month = now.month();

        let result = sqlx::query(
            "DELETE FROM counts WHERE username = ? AND YEAR(txDate) = ? AND MONTH(txDate) = ?"
        )
        .bind(username)
        .bind(year)
        .bind(month)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn delete_last_current_month_expense(&self, username: &str) -> Result<Option<Expense>> {
        use chrono::Local;

        let now = Local::now().date_naive();
        let year = now.year();
        let month = now.month();

        // First, find the most recent expense
        let expense = sqlx::query_as::<_, Expense>(
            "SELECT id, txDate, username, quantity FROM counts 
             WHERE username = ? AND YEAR(txDate) = ? AND MONTH(txDate) = ? 
             ORDER BY txDate DESC, id DESC LIMIT 1"
        )
        .bind(username)
        .bind(year)
        .bind(month)
        .fetch_optional(&self.pool)
        .await?;

        // If found, delete it
        if let Some(ref exp) = expense {
            sqlx::query("DELETE FROM counts WHERE id = ?")
                .bind(exp.id)
                .execute(&self.pool)
                .await?;
        }

        Ok(expense)
    }

    async fn get_year_summary(&self, username: &str, year: i32) -> Result<Vec<(u32, Decimal)>> {
        let results: Vec<(u32, Decimal)> = sqlx::query_as(
            "SELECT MONTH(txDate) as month, SUM(quantity) as total 
             FROM counts 
             WHERE username = ? AND YEAR(txDate) = ? 
             GROUP BY MONTH(txDate) 
             ORDER BY month ASC"
        )
        .bind(username)
        .bind(year)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn get_all_chat_ids(&self) -> Result<Vec<i64>> {
        let chat_ids: Vec<(i64,)> = sqlx::query_as("SELECT DISTINCT chatId FROM config")
            .fetch_all(&self.pool)
            .await?;

        Ok(chat_ids.into_iter().map(|(id,)| id).collect())
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

        async fn get_current_month_expenses(&self, username: &str) -> Result<Vec<Expense>> {
            use chrono::Local;

            let now = Local::now().date_naive();
            let year = now.year();
            let month = now.month();

            let expenses = self.expenses.lock().unwrap();
            let mut result: Vec<Expense> = expenses
                .iter()
                .filter(|e| {
                    e.username == username
                        && e.tx_date.year() == year
                        && e.tx_date.month() == month
                })
                .cloned()
                .collect();

            // Sort by date ascending, then by ID descending
            result.sort_by(|a, b| {
                a.tx_date
                    .cmp(&b.tx_date)
                    .then_with(|| b.id.cmp(&a.id))
            });

            Ok(result)
        }

        async fn delete_current_month_expenses(&self, username: &str) -> Result<u64> {
            use chrono::Local;

            let now = Local::now().date_naive();
            let year = now.year();
            let month = now.month();

            let mut expenses = self.expenses.lock().unwrap();
            let initial_len = expenses.len();

            expenses.retain(|e| {
                !(e.username == username
                    && e.tx_date.year() == year
                    && e.tx_date.month() == month)
            });

            let deleted_count = (initial_len - expenses.len()) as u64;
            Ok(deleted_count)
        }

        async fn delete_last_current_month_expense(&self, username: &str) -> Result<Option<Expense>> {
            use chrono::Local;

            let now = Local::now().date_naive();
            let year = now.year();
            let month = now.month();

            let mut expenses = self.expenses.lock().unwrap();

            // Find the most recent expense in the current month
            let mut current_month_expenses: Vec<&Expense> = expenses
                .iter()
                .filter(|e| {
                    e.username == username
                        && e.tx_date.year() == year
                        && e.tx_date.month() == month
                })
                .collect();

            // Sort by date descending, then by ID descending
            current_month_expenses.sort_by(|a, b| {
                b.tx_date
                    .cmp(&a.tx_date)
                    .then_with(|| b.id.cmp(&a.id))
            });

            // Get the first one (most recent)
            if let Some(most_recent) = current_month_expenses.first() {
                let expense_to_delete = (*most_recent).clone();
                let id_to_delete = expense_to_delete.id;

                // Remove it from the expenses vector
                expenses.retain(|e| e.id != id_to_delete);

                Ok(Some(expense_to_delete))
            } else {
                Ok(None)
            }
        }

        async fn get_year_summary(&self, username: &str, year: i32) -> Result<Vec<(u32, Decimal)>> {
            use std::collections::HashMap;

            let expenses = self.expenses.lock().unwrap();

            // Group expenses by month and sum them
            let mut monthly_totals: HashMap<u32, Decimal> = HashMap::new();

            for expense in expenses.iter() {
                if expense.username == username && expense.tx_date.year() == year {
                    let month = expense.tx_date.month();
                    *monthly_totals.entry(month).or_insert(Decimal::ZERO) += expense.quantity;
                }
            }

            // Convert to vector and sort by month
            let mut result: Vec<(u32, Decimal)> = monthly_totals.into_iter().collect();
            result.sort_by_key(|(month, _)| *month);

            Ok(result)
        }

        async fn get_all_chat_ids(&self) -> Result<Vec<i64>> {
            let users = self.users.lock().unwrap();
            let mut chat_ids: Vec<i64> = users.values().map(|u| u.chat_id).collect();
            
            // Remove duplicates and sort for consistency
            chat_ids.sort_unstable();
            chat_ids.dedup();

            Ok(chat_ids)
        }
    }
}

#[cfg(test)]
#[path = "repository_test.rs"]
mod repository_test;
