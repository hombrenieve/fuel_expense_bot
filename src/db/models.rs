// Database models
// Will be implemented in task 4.1

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct UserConfig {
    pub username: String,
    #[sqlx(rename = "chatId")]
    pub chat_id: i64,
    #[sqlx(rename = "payLimit")]
    pub pay_limit: Decimal,
}

#[derive(Debug, Clone, FromRow)]
pub struct Expense {
    pub id: i64,
    #[sqlx(rename = "txDate")]
    pub tx_date: NaiveDate,
    pub username: String,
    pub quantity: Decimal,
}

#[derive(Debug, Clone)]
pub struct MonthlySummary {
    pub total_spent: Decimal,
    pub limit: Decimal,
    pub remaining: Decimal,
}

#[derive(Debug, Clone)]
pub enum ExpenseAddResult {
    Created(i64),
    Updated(i64),
    LimitExceeded { current: Decimal, limit: Decimal },
}
