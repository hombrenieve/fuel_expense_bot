# Design Document: Enhanced Expense Management

## Overview

This design extends the Telegram fuel bot with enhanced expense management capabilities and version notification features. The enhancements provide users with detailed expense views, selective deletion operations with historical data protection, yearly summaries, and automatic notifications when new bot versions are deployed.

The design follows the existing architecture pattern with:
- Repository layer for database operations
- Service layer for business logic
- Bot command handlers for user interaction
- Version tracking mechanism for deployment notifications

## Architecture

### System Components

The feature integrates into the existing three-tier architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    Bot Layer                             │
│  (Command Handlers: /list_month, /clear_month, etc.)   │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                  Service Layer                           │
│  (ExpenseService: business logic & validation)          │
│  (VersionService: version tracking & notifications)     │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                 Repository Layer                         │
│  (Database operations: queries, transactions)           │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   MySQL Database                         │
│  (Tables: counts, config, version_tracking)             │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Expense Listing**: User → Bot Handler → ExpenseService → Repository → Database
2. **Expense Deletion**: User → Bot Handler → ExpenseService (validates current month) → Repository → Database
3. **Year Summary**: User → Bot Handler → ExpenseService → Repository (aggregates) → Database
4. **Version Notification**: Bot Startup → VersionService → Repository (get chats) → Bot (send messages)

## Components and Interfaces

### 1. Repository Layer Extensions

The repository layer will be extended with new methods for the enhanced operations.

#### New Repository Methods

```rust
// In src/db/repository.rs

#[async_trait]
pub trait RepositoryTrait: Send + Sync {
    // ... existing methods ...
    
    /// Get all expenses for a user in the current month with detailed information
    /// Returns expenses ordered chronologically by date
    async fn get_current_month_expenses(&self, username: &str) -> Result<Vec<Expense>>;
    
    /// Delete all expenses for a user in the current month
    /// Returns the number of expenses deleted
    async fn delete_current_month_expenses(&self, username: &str) -> Result<u64>;
    
    /// Delete the most recent expense for a user in the current month
    /// Returns the deleted expense if one existed, None otherwise
    async fn delete_last_current_month_expense(&self, username: &str) -> Result<Option<Expense>>;
    
    /// Get monthly totals for the entire current year
    /// Returns a vector of (month, total) tuples for months with expenses
    async fn get_year_summary(&self, username: &str, year: i32) -> Result<Vec<(u32, Decimal)>>;
    
    /// Get all active chat IDs for startup notifications
    /// Returns a list of unique chat IDs from the config table
    async fn get_all_chat_ids(&self) -> Result<Vec<i64>>;
}
```

#### Implementation Details

**get_current_month_expenses**:
- Query: `SELECT * FROM counts WHERE username = ? AND YEAR(txDate) = ? AND MONTH(txDate) = ? ORDER BY txDate ASC, id DESC`
- Uses current year/month from system date
- Orders by date ascending, then by ID descending (for same-day expenses, most recent first)

**delete_current_month_expenses**:
- Query: `DELETE FROM counts WHERE username = ? AND YEAR(txDate) = ? AND MONTH(txDate) = ?`
- Uses current year/month to ensure only current month is affected
- Returns rows_affected count

**delete_last_current_month_expense**:
- First query: `SELECT * FROM counts WHERE username = ? AND YEAR(txDate) = ? AND MONTH(txDate) = ? ORDER BY txDate DESC, id DESC LIMIT 1`
- If found, delete query: `DELETE FROM counts WHERE id = ?`
- Returns the expense that was deleted for confirmation message

**get_year_summary**:
- Query: `SELECT MONTH(txDate) as month, SUM(quantity) as total FROM counts WHERE username = ? AND YEAR(txDate) = ? GROUP BY MONTH(txDate) ORDER BY month ASC`
- Returns vector of (month_number, total) tuples
- Months with no expenses are omitted from results

**Version tracking methods**:
- get_all_chat_ids: `SELECT DISTINCT chatId FROM config`

### 2. Service Layer Extensions

#### ExpenseService Extensions

```rust
// In src/services/expense_service.rs

impl ExpenseService {
    /// Get detailed list of current month's expenses
    /// Returns expenses with day information for display
    pub async fn list_current_month_expenses(&self, username: &str) -> Result<Vec<ExpenseDetail>>;
    
    /// Clear all expenses from the current month
    /// Returns the count of deleted expenses
    pub async fn clear_current_month(&self, username: &str) -> Result<u64>;
    
    /// Remove the last (most recent) expense from the current month
    /// Returns the deleted expense details if one existed
    pub async fn remove_last_expense(&self, username: &str) -> Result<Option<ExpenseDetail>>;
    
    /// Get summary of expenses for the entire current year
    /// Returns monthly totals and a grand total
    pub async fn get_year_summary(&self, username: &str) -> Result<YearSummary>;
}

#[derive(Debug, Clone)]
pub struct ExpenseDetail {
    pub day: u32,           // Day of month (1-31)
    pub amount: Decimal,    // Expense amount
    pub date: NaiveDate,    // Full date for reference
}

#[derive(Debug, Clone)]
pub struct YearSummary {
    pub year: i32,
    pub monthly_totals: Vec<MonthTotal>,
    pub grand_total: Decimal,
}

#[derive(Debug, Clone)]
pub struct MonthTotal {
    pub month: u32,         // Month number (1-12)
    pub month_name: String, // Month name for display
    pub total: Decimal,
}
```

#### VersionService (New Service)

```rust
// In src/services/version_service.rs

use std::sync::Arc;
use crate::db::repository::RepositoryTrait;
use crate::utils::error::Result;

pub struct VersionService {
    repo: Arc<dyn RepositoryTrait>,
}

impl VersionService {
    pub fn new(repo: Arc<dyn RepositoryTrait>) -> Self {
        Self { repo }
    }
    
    /// Get all chat IDs that should receive startup notifications
    pub async fn get_notification_targets(&self) -> Result<Vec<i64>>;
    
    /// Get the current version from Cargo.toml
    pub fn get_current_version() -> &'static str;
    
    /// Get the change description from Cargo.toml metadata
    pub fn get_change_description() -> Result<String>;
}
```

### 3. Bot Command Handlers

#### New Commands

```rust
// In src/bot/handlers.rs or similar

/// Handler for /list_month command
/// Shows all expenses in the current month with day information
async fn handle_list_month(bot: Bot, msg: Message, expense_service: Arc<ExpenseService>) -> ResponseResult<()>;

/// Handler for /year_summary command  
/// Shows monthly totals for the current year
async fn handle_year_summary(bot: Bot, msg: Message, expense_service: Arc<ExpenseService>) -> ResponseResult<()>;

/// Handler for /clear_month command
/// Removes all expenses from the current month with confirmation
async fn handle_clear_month(bot: Bot, msg: Message, expense_service: Arc<ExpenseService>) -> ResponseResult<()>;

/// Handler for /remove_last command
/// Removes the most recent expense from the current month
async fn handle_remove_last(bot: Bot, msg: Message, expense_service: Arc<ExpenseService>) -> ResponseResult<()>;
```

#### Startup Notification

```rust
// In src/main.rs

async fn send_startup_notifications(
    bot: Bot,
    version_service: Arc<VersionService>,
) -> Result<()> {
    // Get current version and change description
    let current_version = VersionService::get_current_version();
    let changes = VersionService::get_change_description()?;
    
    // Get all chat IDs
    let chat_ids = version_service.get_notification_targets().await?;
    
    // Send notification to each chat
    let message = format!(
        "🔔 Bot started - Version {}\n\nChanges:\n{}",
        current_version, changes
    );
    
    for chat_id in chat_ids {
        let _ = bot.send_message(ChatId(chat_id), &message).await;
    }
    
    Ok(())
}
```

## Data Models

### Database Schema Changes

No new tables are required. All operations work with the existing `counts` and `config` tables.

### Cargo.toml Metadata Extension

```toml
[package]
name = "telegram-fuel-bot"
version = "0.2.0"
edition = "2021"

[package.metadata.changelog]
description = """
- Added detailed expense listing for current month
- Added year summary with monthly totals
- Added ability to clear current month expenses
- Added ability to remove last expense
- Added version change notifications
"""
```

The change description will be read from `package.metadata.changelog.description` field.

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Property 1: Current Month Expense Retrieval Completeness
*For any* set of expenses in the current month, when listing current month expenses, all expenses recorded in the current month should be returned and no expenses from other months should be included.
**Validates: Requirements 1.1**

### Property 2: Expense Display Completeness
*For any* expense in the current month, when displaying that expense, the output should contain both the day of the month and the expense amount.
**Validates: Requirements 1.2, 1.3**

### Property 3: Chronological Ordering of Expenses
*For any* set of expenses in the current month, when listing expenses, they should be ordered chronologically by recording date (earliest to latest).
**Validates: Requirements 1.5**

### Property 4: Year Summary Completeness
*For any* set of expenses in the current year, when generating a year summary, all months that have expenses should be represented with correct totals.
**Validates: Requirements 2.1**

### Property 5: Monthly Total Display Format
*For any* month with expenses in the year summary, the display should include both the month name and the total amount.
**Validates: Requirements 2.2**

### Property 6: Year Summary Chronological Ordering
*For any* year summary, the monthly totals should be ordered chronologically from January (1) to December (12).
**Validates: Requirements 2.4**

### Property 7: Year Summary Grand Total Accuracy
*For any* year summary, the grand total should equal the sum of all monthly totals.
**Validates: Requirements 2.5**

### Property 8: Clear Current Month Completeness
*For any* set of expenses in the current month, when clearing current month expenses, all current month expenses should be removed and the operation should return the count of removed expenses.
**Validates: Requirements 3.1, 3.3**

### Property 9: Previous Month Protection During Clear
*For any* set of expenses spanning current and previous months, when clearing current month expenses, all previous month expenses should remain unchanged.
**Validates: Requirements 3.2**

### Property 10: Last Expense Identification
*For any* set of expenses in the current month, when removing the last expense, the expense with the most recent date (and latest timestamp if multiple on same day) should be identified and removed.
**Validates: Requirements 4.1, 4.2, 4.4**

### Property 11: Remove Last Scope Restriction
*For any* set of expenses spanning current and previous months, when removing the last expense, only current month expenses should be considered for removal.
**Validates: Requirements 5.3**

### Property 12: Chat ID Retrieval for Notifications
*For any* set of user configurations, when retrieving notification targets, all unique chat IDs should be returned.
**Validates: Requirements 6.1**

### Property 13: Startup Notification Message Completeness
*For any* startup notification, the message should include both the current version number and the change description from Cargo.toml metadata.
**Validates: Requirements 6.3, 6.4**

## Error Handling

### Input Validation

1. **User Existence**: All operations require a valid user. Return `BotError::UserNotFound` if user doesn't exist.
2. **Amount Validation**: Expense amounts must be positive decimals (handled by existing validation).
3. **Date Validation**: All date operations use system-provided dates to ensure validity.

### Database Errors

1. **Connection Failures**: Propagate database connection errors with context.
2. **Query Failures**: Wrap SQL errors in `BotError::Database` with descriptive messages.
3. **Transaction Failures**: Ensure proper rollback on transaction errors.

### Startup Notification Errors

1. **Missing Metadata**: If Cargo.toml metadata is missing or malformed, use default message "Bot started. See release notes for details."
2. **Chat Send Failures**: Log individual chat send failures but continue with remaining chats.
3. **Empty Chat List**: If no chats are configured, skip notifications gracefully.

### Edge Cases

1. **Empty Current Month**: Return appropriate "no expenses" messages for list, clear, and remove operations.
2. **Same-Day Multiple Expenses**: Use timestamp ordering (ID as tiebreaker) for "last expense" determination.
3. **Year with No Expenses**: Return empty summary with zero grand total.
4. **No Active Chats**: If no users are configured, skip startup notifications gracefully.

## Testing Strategy

### Dual Testing Approach

This feature will use both unit tests and property-based tests to ensure comprehensive coverage:

- **Unit tests**: Verify specific examples, edge cases (empty month, first run), and error conditions
- **Property tests**: Verify universal properties across all inputs using randomized data

Together, these approaches provide comprehensive coverage where unit tests catch concrete bugs and property tests verify general correctness.

### Property-Based Testing

Property-based tests will be implemented using the `proptest` crate (already in dev-dependencies). Each correctness property listed above will be implemented as a property-based test.

**Configuration**:
- Minimum 100 iterations per property test
- Each test tagged with: `Feature: enhanced-expense-management, Property {number}: {property_text}`

**Test Data Generators**:
- Random expense amounts (positive decimals)
- Random dates (current month, previous months, various years)
- Random usernames and chat IDs
- Random version strings (semantic versioning format)

**Example Property Test Structure**:

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    
    // Feature: enhanced-expense-management, Property 1: Current Month Expense Retrieval Completeness
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn test_current_month_retrieval_completeness(
            current_month_expenses in prop::collection::vec(expense_strategy(), 0..20),
            other_month_expenses in prop::collection::vec(expense_strategy(), 0..20)
        ) {
            // Test that listing returns all and only current month expenses
        }
    }
}
```

### Unit Testing

Unit tests will focus on:

1. **Specific Examples**:
   - List expenses with known data
   - Clear month with specific expense counts
   - Remove last with known most recent expense

2. **Edge Cases**:
   - Empty current month (Requirements 1.4, 3.4, 4.3)
   - Empty year (Requirement 2.3)
   - Multiple expenses on same day with different timestamps
   - No active chats for startup notifications

3. **Error Conditions**:
   - User not found errors
   - Database connection failures
   - Missing Cargo.toml metadata
   - Invalid version format

4. **Integration Points**:
   - Repository method interactions
   - Service layer business logic
   - Command handler message formatting

### Test Organization

```
tests/
├── unit/
│   ├── expense_service_enhanced_test.rs
│   ├── version_service_test.rs
│   └── repository_enhanced_test.rs
└── property/
    ├── expense_operations_properties.rs
    ├── year_summary_properties.rs
    └── version_tracking_properties.rs
```

### Mock Repository

The existing `MockRepository` will be extended to support the new repository methods, enabling service-layer testing without database dependencies.

## Implementation Notes

### Month Boundary Handling

All "current month" operations use the system date at the time of execution:
- `chrono::Local::now().date_naive()` provides the current date
- Extract year and month: `date.year()` and `date.month()`
- SQL queries use `YEAR(txDate) = ? AND MONTH(txDate) = ?` for month filtering

### Timestamp Ordering

For same-day expenses, the database ID serves as a tiebreaker:
- Higher ID = more recent (auto-increment primary key)
- Query: `ORDER BY txDate DESC, id DESC LIMIT 1`

### Version Metadata Parsing

The change description is read from Cargo.toml at compile time:
```rust
const VERSION: &str = env!("CARGO_PKG_VERSION");
const CHANGELOG: &str = env!("CARGO_PKG_METADATA_CHANGELOG_DESCRIPTION");
```

If metadata is missing, use a default message: "Version updated. See release notes for details."

### Notification Timing

Startup notifications are sent during bot startup:
1. Load current version from Cargo.toml
2. Load change description from Cargo.toml metadata
3. Get all active chat IDs from database
4. Send notification to each chat
5. Continue with normal bot operation

This serves as both a version notification and a service restart indicator.
