# Implementation Plan: Rust Telegram Bot Migration

## Overview

This plan outlines the implementation steps for migrating the Telegram fuel expense bot from JavaScript to Rust. The implementation follows a bottom-up approach, starting with foundational utilities and data structures, then building up to business logic, and finally integrating the bot interface.

## Tasks

- [x] 1. Project setup and dependencies
  - Initialize Rust project with Cargo
  - Add dependencies: teloxide, sqlx (with mysql feature), tokio, serde, rust_decimal, chrono, thiserror, tracing, proptest
  - Create module structure (config, db, services, bot, utils)
  - Set up basic logging configuration
  - _Requirements: 7.5, 8.1, 8.2, 8.3_

- [ ] 2. Implement error handling and utilities
  - [x] 2.1 Create error types in `utils/error.rs`
    - Define BotError enum with variants for Database, Config, InvalidInput, UserNotFound, Telegram, Parse errors
    - Implement Display and Error traits
    - Add From conversions for sqlx::Error and teloxide::RequestError
    - _Requirements: 7.1, 7.2, 7.3_
  
  - [x] 2.2 Implement date utilities in `utils/date.rs`
    - Implement `current_date()` function
    - Implement `get_month_bounds(year, month)` function
    - Implement `current_month_bounds()` function
    - Implement `format_date_for_db(date)` function
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
  
  - [ ]* 2.3 Write property tests for date utilities
    - **Property 11: Month Boundary Isolation**
    - **Property 12: Date Serialization Round Trip**
    - Generate random dates and verify month boundary calculations
    - Verify date formatting round trips correctly
    - _Requirements: 6.1, 6.2, 6.4_

- [ ] 3. Implement configuration management
  - [x] 3.1 Create configuration structures in `config.rs`
    - Define Config struct with telegram_token, database config, default_limit
    - Define DatabaseConfig struct with connection parameters
    - Implement Deserialize for both structs
    - _Requirements: 8.1, 8.2, 8.3_
  
  - [x] 3.2 Implement configuration loading
    - Implement `load()` function that reads from environment variables and config file
    - Prioritize environment variables over config file values
    - Implement `validate()` function to check required fields
    - Return clear error if required configuration is missing
    - _Requirements: 8.4, 8.5_
  
  - [x] 3.3 Write unit tests for configuration
    - Test loading from environment variables
    - Test loading from config file
    - Test environment variable priority
    - Test validation with missing required fields
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ] 4. Implement database models and repository trait
  - [x] 4.1 Create database models in `db/models.rs`
    - Define UserConfig struct with sqlx::FromRow derive
    - Define Expense struct with sqlx::FromRow derive
    - Define MonthlySummary struct
    - Define ExpenseAddResult enum
    - _Requirements: 1.1, 2.1, 3.1_
  
  - [x] 4.2 Define RepositoryTrait in `db/repository.rs`
    - Define async trait with methods: create_user, get_user_config, update_user_limit
    - Add methods: get_expense_for_date, create_expense, update_expense, get_monthly_total
    - Add method: add_expense_with_limit_check (with transaction support)
    - _Requirements: 1.1, 2.1, 3.1, 4.1, 5.1, 5.2_
  
  - [-] 4.3 Implement MockRepository for testing
    - Create MockRepository struct with Arc<Mutex<HashMap>> for users and expenses
    - Implement RepositoryTrait for MockRepository using in-memory operations
    - Simulate database constraints (unique username, unique user+date for expenses)
    - _Requirements: 10.1, 10.7_

- [ ] 5. Implement real database repository
  - [~] 5.1 Create connection pool management in `db/pool.rs`
    - Implement function to create MySqlPool from DatabaseConfig
    - Configure connection pool settings (max_connections, timeouts)
    - _Requirements: 5.5_
  
  - [~] 5.2 Implement Repository struct with sqlx
    - Create Repository struct wrapping MySqlPool
    - Implement RepositoryTrait for Repository
    - Implement create_user with INSERT query
    - Implement get_user_config with SELECT query
    - Implement update_user_limit with UPDATE query
    - _Requirements: 1.1, 1.2, 4.2, 5.3, 5.4_
  
  - [~] 5.3 Implement expense operations with transactions
    - Implement get_expense_for_date with SELECT query
    - Implement create_expense with INSERT query
    - Implement update_expense with UPDATE query
    - Implement get_monthly_total with SELECT SUM query filtering by month bounds
    - Implement add_expense_with_limit_check using sqlx transactions
    - _Requirements: 2.1, 2.2, 3.1, 5.1, 5.2, 6.1_

- [~] 6. Checkpoint - Ensure database layer tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Implement user service
  - [~] 7.1 Create UserService in `services/user_service.rs`
    - Define UserService struct with Arc<dyn RepositoryTrait> and default_limit
    - Implement register_user function (calls repo.create_user, handles already exists case)
    - Implement update_limit function with validation (reject negative/zero values)
    - Implement get_config function
    - _Requirements: 1.1, 1.2, 1.3, 4.1, 4.2, 4.3_
  
  - [ ]* 7.2 Write property tests for user service
    - **Property 1: User Registration Creates Valid Records**
    - **Property 2: User Registration is Idempotent**
    - **Property 9: Limit Configuration Validation**
    - Generate random usernames and chat IDs, verify registration
    - Test idempotency with multiple registrations
    - Generate random valid/invalid limits, verify validation
    - _Requirements: 1.1, 1.2, 1.3, 4.1, 4.2, 4.3_
  
  - [ ]* 7.3 Write unit tests for user service edge cases
    - Test registering user with empty username
    - Test updating limit to exactly 0.01 (minimum valid)
    - Test updating limit with very large value
    - _Requirements: 1.1, 4.2, 4.3_

- [ ] 8. Implement expense service
  - [~] 8.1 Create ExpenseService in `services/expense_service.rs`
    - Define ExpenseService struct with Arc<dyn RepositoryTrait>
    - Implement add_expense function (gets user config, calls validate_and_add_with_transaction)
    - Implement validate_and_add_with_transaction (uses current_date, calls repo with transaction)
    - Implement get_monthly_summary function (gets monthly total and user config, calculates remaining)
    - _Requirements: 2.2, 2.3, 2.4, 2.5, 2.6, 3.1, 3.2, 3.3, 3.4, 5.1, 5.2_
  
  - [~] 8.2 Write property tests for expense service
    - **Property 4: Expense Date Assignment**
    - **Property 5: Limit Enforcement**
    - **Property 6: Successful Expense Addition**
    - **Property 7: Monthly Total Calculation**
    - **Property 8: Summary Arithmetic Correctness**
    - Generate random users, limits, and expense sequences
    - Verify limit enforcement for all combinations
    - Verify monthly totals are calculated correctly
    - Verify summary arithmetic (remaining = limit - spent)
    - _Requirements: 2.2, 2.3, 2.4, 2.5, 2.6, 3.1, 3.2, 3.3, 3.4, 6.1_
  
  - [ ]* 8.3 Write property tests for concurrency
    - **Property 10: Transaction Atomicity**
    - **Property 13: Concurrent Operation Consistency**
    - **Property 14: User Isolation**
    - Test concurrent expense additions for same user
    - Test concurrent expense additions for different users
    - Verify no lost updates and correct final totals
    - _Requirements: 5.1, 5.2_
  
  - [ ]* 8.4 Write unit tests for expense service edge cases
    - Test adding expense exactly equal to remaining limit
    - Test adding expense of 0.01 (minimum amount)
    - Test monthly summary with no expenses (total should be 0.00)
    - Test same-day expense accumulation
    - _Requirements: 2.3, 2.4, 2.5, 3.1_

- [~] 9. Checkpoint - Ensure business logic tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Implement bot command handlers
  - [~] 10.1 Create command handlers in `bot/handlers.rs`
    - Implement handle_start (extracts username and chat_id, calls user_service.register_user)
    - Implement handle_check (extracts username, calls expense_service.get_monthly_summary, formats response)
    - Implement handle_config (parses command args, validates, calls user_service.update_limit)
    - Implement handle_numeric_input (parses amount, calls expense_service.add_expense, formats response)
    - Add user-friendly error message formatting for all error types
    - _Requirements: 1.1, 1.2, 2.1, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 7.3_
  
  - [ ]* 10.2 Write property tests for command parsing
    - **Property 3: Numeric Input Parsing Round Trip**
    - Generate random valid decimal strings, verify parsing
    - Test edge cases: "0.01", "9999.99", "100"
    - _Requirements: 2.1_
  
  - [ ]* 10.3 Write unit tests for command handlers
    - Test /start with new user
    - Test /start with existing user
    - Test /check with expenses
    - Test /check with no expenses
    - Test /config with valid limit
    - Test /config with invalid limit
    - Test numeric input within limit
    - Test numeric input exceeding limit
    - Test error message formatting (no stack traces, user-friendly)
    - _Requirements: 1.1, 1.2, 2.1, 2.3, 2.4, 3.1, 4.1, 4.2, 7.3_

- [ ] 11. Implement bot dispatcher and main application
  - [~] 11.1 Create bot dispatcher in `bot/dispatcher.rs`
    - Set up teloxide dispatcher with command routing
    - Route /start to handle_start
    - Route /check to handle_check
    - Route /config to handle_config
    - Route text messages to handle_numeric_input (if numeric)
    - Add logging for all incoming commands
    - _Requirements: 7.4_
  
  - [~] 11.2 Implement main application in `main.rs`
    - Load configuration using config::load()
    - Initialize tracing/logging
    - Create database connection pool
    - Create Repository, UserService, ExpenseService instances
    - Initialize teloxide bot with token
    - Set up graceful shutdown handler (SIGTERM, SIGINT)
    - Start bot dispatcher
    - On shutdown: stop accepting commands, complete in-progress operations, close connections, log shutdown
    - _Requirements: 7.1, 7.4, 7.5, 8.1, 8.2, 8.3, 9.1, 9.2, 9.3, 9.4_
  
  - [ ]* 11.3 Write integration tests for bot flow
    - Test full flow: /start → add expense → /check
    - Test graceful shutdown sequence
    - _Requirements: 1.1, 2.4, 3.1, 9.1, 9.2, 9.3, 9.4_

- [~] 12. Add database migrations
  - Create SQL migration files for config and counts tables
  - Ensure schema matches existing JavaScript version for compatibility
  - Add migration runner to main.rs (optional, run on startup or manually)
  - _Requirements: 1.1, 2.1_

- [~] 13. Add documentation and deployment configuration
  - Create README.md with setup instructions
  - Document environment variables and configuration options
  - Add example .env file
  - Create Dockerfile for containerized deployment (optional)
  - Add systemd service file example (optional)
  - _Requirements: 8.1, 8.2, 8.3_

- [~] 14. Final checkpoint - Run full test suite
  - Run all unit tests: `cargo test`
  - Run property tests with high iteration count: `PROPTEST_CASES=1000 cargo test`
  - Verify code compiles without warnings: `cargo clippy`
  - Format code: `cargo fmt`
  - Ensure all tests pass, ask the user if questions arise.

- [~] 15. Clean up JavaScript code
  - Remove or archive the old JavaScript bot.js file
  - Remove or archive the old JavaScript db.js file
  - Remove package.json (if no longer needed)
  - Update any documentation that references the JavaScript implementation
  - _Requirements: N/A (cleanup task)_

## Notes

- Tasks marked with `*` are optional test tasks that can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties across many generated inputs
- Unit tests validate specific examples and edge cases
- The implementation follows a bottom-up approach: utilities → data layer → business logic → bot interface
- MockRepository allows testing without a real database
- All database operations use transactions for atomicity
- Error handling provides user-friendly messages without exposing internals
