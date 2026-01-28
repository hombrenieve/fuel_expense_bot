# Design Document: Rust Telegram Bot Migration

## Overview

This design document outlines the architecture and implementation approach for migrating a Telegram fuel expense tracking bot from JavaScript/Node.js to Rust. The migration will leverage Rust's type safety, performance, and robust error handling while maintaining feature parity with the existing system.

The bot will use:
- **teloxide** for Telegram bot functionality (async, type-safe, actively maintained)
- **sqlx** for database operations (compile-time checked queries, async, connection pooling)
- **tokio** as the async runtime
- **serde** for configuration and serialization
- **tracing** for structured logging
- **proptest** for property-based testing

## Architecture

### High-Level Architecture

```
┌─────────────┐
│   Telegram  │
│   Platform  │
└──────┬──────┘
       │
       │ Bot API (HTTPS)
       │
┌──────▼──────────────────────────────────────┐
│         Teloxide Bot Framework              │
│  ┌────────────────────────────────────┐    │
│  │   Command Handlers                 │    │
│  │  - /start  - /check                │    │
│  │  - /config - numeric input         │    │
│  └────────┬───────────────────────────┘    │
└───────────┼──────────────────────────────────┘
            │
            │
┌───────────▼──────────────────────────────────┐
│         Business Logic Layer                 │
│  ┌──────────────────────────────────────┐   │
│  │  ExpenseService                      │   │
│  │  - validate_and_add_expense()        │   │
│  │  - get_monthly_summary()             │   │
│  │  - check_limit()                     │   │
│  └──────────┬───────────────────────────┘   │
│             │                                │
│  ┌──────────▼───────────────────────────┐   │
│  │  UserService                         │   │
│  │  - register_user()                   │   │
│  │  - update_limit()                    │   │
│  │  - get_user_config()                 │   │
│  └──────────┬───────────────────────────┘   │
└─────────────┼────────────────────────────────┘
              │
              │
┌─────────────▼────────────────────────────────┐
│         Database Layer (sqlx)                │
│  ┌──────────────────────────────────────┐   │
│  │  Database Repository                 │   │
│  │  - Connection Pool                   │   │
│  │  - Transaction Support               │   │
│  │  - Query Methods                     │   │
│  └──────────┬───────────────────────────┘   │
└─────────────┼────────────────────────────────┘
              │
              │
┌─────────────▼────────────────────────────────┐
│         MariaDB/MySQL Database               │
│  ┌──────────────┐  ┌──────────────────┐     │
│  │ config table │  │  counts table    │     │
│  └──────────────┘  └──────────────────┘     │
└──────────────────────────────────────────────┘
```

### Module Structure

```
src/
├── main.rs                 # Application entry point, initialization
├── config.rs               # Configuration management
├── bot/
│   ├── mod.rs             # Bot module exports
│   ├── handlers.rs        # Command handlers
│   └── dispatcher.rs      # Command routing
├── services/
│   ├── mod.rs             # Service layer exports
│   ├── expense_service.rs # Expense business logic
│   └── user_service.rs    # User management logic
├── db/
│   ├── mod.rs             # Database module exports
│   ├── repository.rs      # Database operations
│   ├── models.rs          # Database models
│   └── pool.rs            # Connection pool management
├── utils/
│   ├── mod.rs             # Utility exports
│   ├── date.rs            # Date handling utilities
│   └── error.rs           # Error types and conversions
└── tests/
    ├── integration/       # Integration tests
    └── property/          # Property-based tests
```

## Components and Interfaces

### 1. Configuration Module (`config.rs`)

**Purpose:** Load and validate application configuration from environment variables or config file.

**Data Structure:**
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub telegram_token: String,
    pub database: DatabaseConfig,
    pub default_limit: Decimal,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}
```

**Key Functions:**
- `load() -> Result<Config>` - Load configuration with environment variable priority
- `validate(&self) -> Result<()>` - Validate configuration values

### 2. Database Models (`db/models.rs`)

**Purpose:** Define type-safe representations of database entities.

**Data Structures:**
```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserConfig {
    pub username: String,
    pub chat_id: i64,
    pub pay_limit: Decimal,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Expense {
    pub id: i64,
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
```

### 3. Database Repository (`db/repository.rs`)

**Purpose:** Provide database operations with proper error handling and transaction support.

**Key Functions:**
```rust
pub struct Repository {
    pool: MySqlPool,
}

impl Repository {
    // User operations
    pub async fn create_user(&self, username: &str, chat_id: i64, default_limit: Decimal) 
        -> Result<()>;
    
    pub async fn get_user_config(&self, username: &str) 
        -> Result<Option<UserConfig>>;
    
    pub async fn update_user_limit(&self, username: &str, new_limit: Decimal) 
        -> Result<()>;
    
    // Expense operations
    pub async fn get_expense_for_date(&self, username: &str, date: NaiveDate) 
        -> Result<Option<Expense>>;
    
    pub async fn create_expense(&self, username: &str, date: NaiveDate, amount: Decimal) 
        -> Result<i64>;
    
    pub async fn update_expense(&self, id: i64, new_amount: Decimal) 
        -> Result<()>;
    
    pub async fn get_monthly_total(&self, username: &str, year: i32, month: u32) 
        -> Result<Decimal>;
    
    // Transaction support
    pub async fn add_expense_with_limit_check<'a>(
        &self,
        tx: &mut Transaction<'a, MySql>,
        username: &str,
        date: NaiveDate,
        amount: Decimal,
        limit: Decimal,
    ) -> Result<ExpenseAddResult>;
}

pub enum ExpenseAddResult {
    Created(i64),
    Updated(i64),
    LimitExceeded { current: Decimal, limit: Decimal },
}
```

### 4. User Service (`services/user_service.rs`)

**Purpose:** Handle user-related business logic.

**Key Functions:**
```rust
pub struct UserService {
    repo: Arc<Repository>,
    default_limit: Decimal,
}

impl UserService {
    pub async fn register_user(&self, username: String, chat_id: i64) 
        -> Result<RegistrationResult>;
    
    pub async fn update_limit(&self, username: &str, new_limit: Decimal) 
        -> Result<()>;
    
    pub async fn get_config(&self, username: &str) 
        -> Result<UserConfig>;
}

pub enum RegistrationResult {
    NewUser,
    AlreadyRegistered,
}
```

### 5. Expense Service (`services/expense_service.rs`)

**Purpose:** Handle expense-related business logic with limit enforcement.

**Key Functions:**
```rust
pub struct ExpenseService {
    repo: Arc<Repository>,
}

impl ExpenseService {
    pub async fn add_expense(
        &self,
        username: &str,
        amount: Decimal,
    ) -> Result<AddExpenseResult>;
    
    pub async fn get_monthly_summary(
        &self,
        username: &str,
    ) -> Result<MonthlySummary>;
    
    async fn validate_and_add_with_transaction(
        &self,
        username: &str,
        date: NaiveDate,
        amount: Decimal,
        user_config: &UserConfig,
    ) -> Result<AddExpenseResult>;
}

pub enum AddExpenseResult {
    Success { new_total: Decimal, remaining: Decimal },
    LimitExceeded { current: Decimal, attempted: Decimal, limit: Decimal },
}
```

### 6. Date Utilities (`utils/date.rs`)

**Purpose:** Provide date handling and month calculation utilities.

**Key Functions:**
```rust
pub fn current_date() -> NaiveDate;

pub fn get_month_bounds(year: i32, month: u32) -> (NaiveDate, NaiveDate);

pub fn current_month_bounds() -> (NaiveDate, NaiveDate);

pub fn format_date_for_db(date: NaiveDate) -> String;
```

### 7. Bot Handlers (`bot/handlers.rs`)

**Purpose:** Handle Telegram commands and route to appropriate services.

**Key Functions:**
```rust
pub async fn handle_start(
    bot: Bot,
    msg: Message,
    user_service: Arc<UserService>,
) -> Result<()>;

pub async fn handle_check(
    bot: Bot,
    msg: Message,
    expense_service: Arc<ExpenseService>,
) -> Result<()>;

pub async fn handle_config(
    bot: Bot,
    msg: Message,
    user_service: Arc<UserService>,
    args: Vec<String>,
) -> Result<()>;

pub async fn handle_numeric_input(
    bot: Bot,
    msg: Message,
    expense_service: Arc<ExpenseService>,
    amount: Decimal,
) -> Result<()>;
```

### 8. Error Handling (`utils/error.rs`)

**Purpose:** Define application-specific error types with proper conversions.

**Error Types:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum BotError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Telegram API error: {0}")]
    Telegram(#[from] teloxide::RequestError),
    
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, BotError>;
```

## Data Models

### Database Schema

The existing database schema will be maintained for compatibility:

**config table:**
```sql
CREATE TABLE config (
    username VARCHAR(255) PRIMARY KEY,
    chatId BIGINT NOT NULL,
    payLimit DECIMAL(10, 2) NOT NULL DEFAULT 210.00
);
```

**counts table:**
```sql
CREATE TABLE counts (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    txDate DATE NOT NULL,
    username VARCHAR(255) NOT NULL,
    quantity DECIMAL(10, 2) NOT NULL,
    FOREIGN KEY (username) REFERENCES config(username),
    UNIQUE KEY unique_user_date (username, txDate)
);
```

### Rust Type Mappings

- SQL `VARCHAR(255)` → Rust `String`
- SQL `BIGINT` → Rust `i64`
- SQL `DECIMAL(10, 2)` → Rust `rust_decimal::Decimal` (precise decimal arithmetic)
- SQL `DATE` → Rust `chrono::NaiveDate`

### Data Flow for Add Expense Operation

```
1. User sends "45.50" to Telegram
2. Bot receives message, extracts username and chat_id
3. Parse "45.50" as Decimal
4. Call expense_service.add_expense(username, amount)
5. Service starts database transaction
6. Within transaction:
   a. Get user config (limit)
   b. Get current month total
   c. Check if existing expense for today
   d. If exists: new_total = current_month_total - old_today + new_today
   e. If not exists: new_total = current_month_total + new_today
   f. If new_total > limit: rollback, return LimitExceeded
   g. If new_total <= limit: update/insert expense, commit
7. Return result to handler
8. Handler sends appropriate message to user
```


## Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### Property 1: User Registration Creates Valid Records

*For any* valid username and chat ID, when registering a new user, the system should create a user record with the provided username, chat ID, and the default limit of 210.00.

**Validates: Requirements 1.1, 1.3**

### Property 2: User Registration is Idempotent

*For any* user, registering the same user multiple times should result in exactly one user record in the database, and subsequent registrations should return an "already registered" result.

**Validates: Requirements 1.2**

### Property 3: Numeric Input Parsing Round Trip

*For any* valid decimal string representing a positive monetary amount (e.g., "45.50", "100", "0.01"), parsing it to a Decimal and formatting it back should preserve the numeric value.

**Validates: Requirements 2.1**

### Property 4: Expense Date Assignment

*For any* expense added without an explicit date, the system should assign the current system date as the transaction date.

**Validates: Requirements 2.2**

### Property 5: Limit Enforcement

*For any* user with a monthly limit L and current month total T, when attempting to add an expense E:
- If T + E > L, the system should reject the expense
- If T + E ≤ L, the system should accept the expense

This property applies to both new expenses and accumulated same-day expenses.

**Validates: Requirements 2.3, 2.6**

### Property 6: Successful Expense Addition

*For any* user and valid expense amount within the monthly limit, adding the expense should result in:
- If no expense exists for that date: a new expense record is created
- If an expense exists for that date: the existing expense is updated with the sum of old and new amounts
- The monthly total increases by the added amount
- The operation completes atomically (all-or-nothing)

**Validates: Requirements 2.4, 2.5, 5.1**

### Property 7: Monthly Total Calculation

*For any* set of expenses for a user, the monthly total should equal the sum of all expense quantities where the transaction date falls within the current calendar month boundaries (first day to last day of month).

**Validates: Requirements 3.1, 6.1**

### Property 8: Summary Arithmetic Correctness

*For any* user's monthly summary, the following must hold:
- The summary contains the total spent amount
- The summary contains the monthly limit
- The summary contains the remaining budget
- remaining = limit - total_spent (arithmetic invariant)

**Validates: Requirements 3.2, 3.3, 3.4**

### Property 9: Limit Configuration Validation

*For any* limit configuration command:
- If the amount is a valid positive number, the system should update the user's limit
- If the amount is invalid (negative, zero, or non-numeric), the system should reject the command
- After successful update, the user's stored limit should equal the provided amount

**Validates: Requirements 4.1, 4.2, 4.3, 4.4**

### Property 10: Transaction Atomicity

*For any* expense addition operation, if any step fails (limit check, database write, etc.), the entire operation should be rolled back, leaving the database in its original state with no partial updates.

**Validates: Requirements 5.1, 5.2**

### Property 11: Month Boundary Isolation

*For any* two expenses with dates in different calendar months, they should not affect each other's monthly totals. Specifically, adding an expense in month M should not change the monthly total for month M-1 or M+1.

**Validates: Requirements 6.2**

### Property 12: Date Serialization Round Trip

*For any* valid date, storing it to the database and retrieving it should produce an equivalent date value (year, month, day components match).

**Validates: Requirements 6.4**

### Property 13: Concurrent Operation Consistency

*For any* two concurrent expense addition operations for the same user, the final monthly total should equal the sum of both expenses (assuming both are within limit), regardless of execution order. The system should not lose updates due to race conditions.

**Validates: Requirements 5.2**

### Property 14: User Isolation

*For any* two different users performing expense operations concurrently, the operations should not interfere with each other. Specifically, user A's expense additions should not affect user B's monthly totals, limits, or expense records, even when operations happen simultaneously.

**Validates: Requirements 5.1, 5.2** (implicit multi-user correctness)

## Error Handling

### Error Categories

1. **User Input Errors**
   - Invalid numeric format for expenses
   - Invalid limit values (negative, zero, non-numeric)
   - Malformed commands
   - **Handling:** Return user-friendly error message, log at INFO level

2. **Business Logic Errors**
   - Expense exceeds monthly limit
   - User not registered
   - **Handling:** Return explanatory message to user, log at INFO level

3. **Database Errors**
   - Connection failures
   - Query execution failures
   - Transaction conflicts
   - **Handling:** Return generic error to user, log detailed error at ERROR level, attempt retry for transient errors

4. **Configuration Errors**
   - Missing required configuration
   - Invalid configuration values
   - **Handling:** Fail fast at startup with clear error message

5. **Telegram API Errors**
   - Network failures
   - Rate limiting
   - Invalid bot token
   - **Handling:** Log error, retry with exponential backoff for transient errors

### Error Recovery Strategies

**Database Connection Loss:**
```rust
// Automatic reconnection with exponential backoff
async fn ensure_connection(&self) -> Result<()> {
    let mut retry_delay = Duration::from_secs(1);
    loop {
        match self.pool.acquire().await {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Database connection failed: {}, retrying in {:?}", e, retry_delay);
                tokio::time::sleep(retry_delay).await;
                retry_delay = std::cmp::min(retry_delay * 2, Duration::from_secs(60));
            }
        }
    }
}
```

**Transaction Retry Logic:**
```rust
// Retry transactions on deadlock or timeout
async fn with_retry<F, T>(&self, operation: F) -> Result<T>
where
    F: Fn() -> Future<Output = Result<T>>,
{
    const MAX_RETRIES: u32 = 3;
    for attempt in 1..=MAX_RETRIES {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if is_transient_error(&e) && attempt < MAX_RETRIES => {
                warn!("Transient error on attempt {}: {}", attempt, e);
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

### User-Facing Error Messages

All error messages to users should:
- Be clear and actionable
- Not expose internal implementation details
- Not include stack traces or technical jargon
- Suggest next steps when appropriate

Examples:
- ❌ "sqlx::Error: Connection refused (os error 111)"
- ✅ "Unable to process your request right now. Please try again in a moment."

- ❌ "ParseDecimalError: invalid digit found in string"
- ✅ "Please enter a valid amount (e.g., 45.50)"

- ❌ "Transaction rolled back due to limit check failure"
- ✅ "This expense would exceed your monthly limit of €210.00. You have €45.30 remaining."

## Testing Strategy

### Overview

The testing strategy employs a dual approach combining unit tests for specific scenarios and property-based tests for universal correctness guarantees. This comprehensive approach ensures both concrete examples work correctly and general properties hold across all inputs.

### Property-Based Testing

**Library:** proptest (Rust's leading property-based testing framework)

**Configuration:**
- Minimum 100 iterations per property test (configurable via environment variable)
- Each test tagged with format: `Feature: rust-telegram-bot-migration, Property {N}: {property_text}`
- Shrinking enabled to find minimal failing examples

**Property Test Coverage:**

1. **User Registration Properties** (Properties 1-2)
   - Generate random usernames (alphanumeric, 1-50 chars) and chat IDs
   - Verify registration creates correct records
   - Verify idempotency across multiple registrations

2. **Numeric Parsing Properties** (Property 3)
   - Generate random valid decimal strings
   - Test round-trip parsing and formatting
   - Include edge cases: very small amounts (0.01), large amounts (9999.99), whole numbers

3. **Limit Enforcement Properties** (Properties 5-6)
   - Generate random users with random limits (50.00 to 500.00)
   - Generate random expense sequences
   - Verify limit checks work correctly for all combinations
   - Verify successful additions update totals correctly

4. **Date and Month Calculation Properties** (Properties 7, 11-12)
   - Generate random dates across multiple years
   - Verify monthly totals only include expenses from correct month
   - Verify month boundaries are respected
   - Test date serialization round trips

5. **Arithmetic Properties** (Property 8)
   - Generate random expense sets
   - Verify summary calculations are always correct
   - Verify remaining = limit - spent invariant

6. **Configuration Properties** (Property 9)
   - Generate random valid and invalid limit values
   - Verify validation works correctly
   - Verify updates persist correctly

7. **Concurrency Properties** (Property 13)
   - Generate random concurrent operation sequences
   - Verify no lost updates
   - Verify final state is consistent

### Unit Testing

Unit tests complement property tests by covering:

**Specific Examples:**
- Registering user "alice" with chat ID 12345
- Adding expense of exactly 45.50
- Checking summary with known expense set

**Edge Cases:**
- Empty expense list (monthly total should be 0.00)
- Expense exactly equal to remaining limit
- Very small amounts (0.01)
- Maximum decimal precision

**Error Conditions:**
- Database connection failure (using mock)
- Invalid input formats
- Missing user scenarios
- Transaction rollback scenarios

**Integration Tests:**
- Full command flow: /start → add expense → /check
- Configuration loading from environment
- Graceful shutdown sequence

### Test Organization

```
tests/
├── unit/
│   ├── date_utils_test.rs
│   ├── expense_service_test.rs
│   ├── user_service_test.rs
│   ├── repository_test.rs
│   └── config_test.rs
├── property/
│   ├── registration_properties.rs
│   ├── expense_properties.rs
│   ├── date_properties.rs
│   └── arithmetic_properties.rs
└── integration/
    ├── bot_commands_test.rs
    ├── database_integration_test.rs
    └── shutdown_test.rs
```

### Test Database Strategy

**Approach:** Mock the Repository trait for unit and property tests

Instead of using a real test database, we'll define a `RepositoryTrait` that the real `Repository` implements, and create a `MockRepository` for testing. This approach:
- Avoids testing sqlx library internals
- Makes tests faster (no database I/O)
- Simplifies test setup and teardown
- Allows precise control over test scenarios

```rust
#[async_trait]
pub trait RepositoryTrait: Send + Sync {
    async fn create_user(&self, username: &str, chat_id: i64, default_limit: Decimal) 
        -> Result<()>;
    async fn get_user_config(&self, username: &str) 
        -> Result<Option<UserConfig>>;
    // ... other methods
}

// Real implementation
impl RepositoryTrait for Repository {
    // Uses sqlx for actual database operations
}

// Mock implementation for tests
pub struct MockRepository {
    users: Arc<Mutex<HashMap<String, UserConfig>>>,
    expenses: Arc<Mutex<Vec<Expense>>>,
}

impl RepositoryTrait for MockRepository {
    // In-memory implementation that simulates database behavior
    async fn create_user(&self, username: &str, chat_id: i64, default_limit: Decimal) 
        -> Result<()> {
        let mut users = self.users.lock().unwrap();
        if users.contains_key(username) {
            return Err(BotError::Database(/* duplicate key error */));
        }
        users.insert(username.to_string(), UserConfig {
            username: username.to_string(),
            chat_id,
            pay_limit: default_limit,
        });
        Ok(())
    }
    // ... other methods with in-memory HashMap/Vec operations
}
```

**For Integration Tests:**
- A small number of integration tests can use a real database to verify SQL queries
- These are optional and separate from the main test suite
- Most testing happens with mocks for speed and simplicity

### Coverage Goals

- **Business Logic:** Minimum 80% code coverage
- **Database Operations:** 100% coverage (critical for data integrity)
- **Error Handling Paths:** 100% coverage (ensure all errors are handled)
- **Command Handlers:** 90% coverage (excluding Telegram API mocking complexity)

### Continuous Integration

All tests must pass before merging:
```bash
# Run all tests
cargo test --all-features

# Run property tests with more iterations
PROPTEST_CASES=1000 cargo test --test property_tests

# Check coverage
cargo tarpaulin --out Html --output-dir coverage
```
