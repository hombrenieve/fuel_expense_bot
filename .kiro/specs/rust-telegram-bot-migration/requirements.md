# Requirements Document

## Introduction

This document specifies the requirements for migrating a Telegram fuel expense tracking bot from JavaScript/Node.js to Rust. The bot helps users track monthly fuel expenses and enforce spending limits through a Telegram interface.

## Glossary

- **Bot**: The Telegram bot application that processes user commands and manages fuel expense tracking
- **User**: A person interacting with the Bot through Telegram
- **Expense**: A fuel purchase transaction with a date and monetary amount
- **Monthly_Limit**: The maximum amount a User can spend on fuel in a calendar month
- **Database**: The MariaDB/MySQL database storing user configurations and expense records
- **Current_Month**: The calendar month containing today's date
- **Transaction**: A database operation that must complete atomically

## Requirements

### Requirement 1: User Registration

**User Story:** As a new user, I want to register with the bot using the /start command, so that I can begin tracking my fuel expenses.

#### Acceptance Criteria

1. WHEN a User sends the /start command, THE Bot SHALL create a new user record with their Telegram username and chat ID
2. WHEN a User sends the /start command and already exists, THE Bot SHALL acknowledge the existing registration
3. WHEN creating a new user record, THE Bot SHALL set the Monthly_Limit to a default value of 210.00
4. IF the Database connection fails during registration, THEN THE Bot SHALL return an error message to the User

### Requirement 2: Expense Recording

**User Story:** As a user, I want to record fuel expenses by sending numeric amounts, so that I can track my spending throughout the month.

#### Acceptance Criteria

1. WHEN a User sends a valid numeric amount (e.g., "45.50"), THE Bot SHALL parse it as a fuel expense
2. WHEN recording an expense, THE Bot SHALL use the current date as the transaction date
3. WHEN an expense would exceed the User's Monthly_Limit, THE Bot SHALL reject the transaction and notify the User
4. WHEN an expense is within the Monthly_Limit, THE Bot SHALL store it in the Database and confirm to the User
5. WHEN a User attempts to record a second expense for the same date and an expense already exists, If the new expense plus the previous one are within the Monthly_limit, THE Bot SHALL update the expense for that date adding up the new expense to the existing one
6. WHEN a User attempts to record a second expense for the same date and an expense already exists, If the new expense plus the previous one are above the Monthly_limit, THE Bot SHALL reject the transaction and notify the User
7. WHEN a User attempts to record an expense for a date with no existing expense, THE Bot SHALL create the new expense record
8. IF the Database operation fails during expense recording, THEN THE Bot SHALL return an error message to the User

### Requirement 3: Spending Summary

**User Story:** As a user, I want to check my current month's spending with the /check command, so that I can monitor my budget usage.

#### Acceptance Criteria

1. WHEN a User sends the /check command, THE Bot SHALL calculate the total expenses for the Current_Month
2. WHEN displaying the summary, THE Bot SHALL show the total spent amount
3. WHEN displaying the summary, THE Bot SHALL show the Monthly_Limit
4. WHEN displaying the summary, THE Bot SHALL show the remaining budget (Monthly_Limit minus total spent)
5. IF the Database query fails, THEN THE Bot SHALL return an error message to the User

### Requirement 4: Limit Configuration

**User Story:** As a user, I want to configure my monthly spending limit using the /config command, so that I can customize the bot to my budget.

#### Acceptance Criteria

1. WHEN a User sends "/config limit <amount>", THE Bot SHALL parse the amount as the new Monthly_Limit
2. WHEN the amount is a valid positive number, THE Bot SHALL update the User's Monthly_Limit in the Database
3. WHEN the amount is invalid (negative, zero, or non-numeric), THE Bot SHALL reject the command and notify the User
4. WHEN the limit is successfully updated, THE Bot SHALL confirm the new limit to the User
5. IF the Database update fails, THEN THE Bot SHALL return an error message to the User

### Requirement 5: Database Operations

**User Story:** As a system administrator, I want reliable database operations with proper transaction support, so that data integrity is maintained.

#### Acceptance Criteria

1. WHEN adding an expense, THE Bot SHALL use a database Transaction to ensure atomicity
2. WHEN checking if an expense exceeds the limit, THE Bot SHALL calculate the current month total and compare it atomically
3. WHEN the Database connection is lost, THE Bot SHALL attempt to reconnect automatically
4. WHEN the Database connection cannot be established, THE Bot SHALL log the error and continue attempting reconnection
5. THE Bot SHALL use connection pooling for efficient database access

### Requirement 6: Date and Time Handling

**User Story:** As a user, I want my expenses tracked by calendar month, so that my budget resets at the beginning of each month.

#### Acceptance Criteria

1. WHEN calculating monthly totals, THE Bot SHALL include all expenses from the first day to the last day of the Current_Month
2. WHEN a new month begins, THE Bot SHALL automatically start tracking expenses for the new month
3. THE Bot SHALL use the system timezone for determining the current date
4. WHEN storing transaction dates, THE Bot SHALL use the date format compatible with the Database schema

### Requirement 7: Error Handling and Logging

**User Story:** As a system administrator, I want comprehensive error handling and structured logging, so that I can diagnose and resolve issues quickly.

#### Acceptance Criteria

1. WHEN any error occurs, THE Bot SHALL log the error with structured context (timestamp, user, operation)
2. WHEN a Database error occurs, THE Bot SHALL log the specific error details
3. WHEN a User receives an error message, THE Bot SHALL provide a user-friendly explanation
4. THE Bot SHALL log all incoming commands and their outcomes for audit purposes
5. THE Bot SHALL use log levels (debug, info, warn, error) appropriately

### Requirement 8: Configuration Management

**User Story:** As a system administrator, I want to configure the bot through environment variables or a config file, so that I can deploy it in different environments without code changes.

#### Acceptance Criteria

1. THE Bot SHALL read the Telegram bot token from configuration
2. THE Bot SHALL read Database connection parameters (host, port, username, password, database name) from configuration
3. THE Bot SHALL read the default Monthly_Limit from configuration
4. WHERE environment variables are provided, THE Bot SHALL prioritize them over config file values
5. IF required configuration is missing, THEN THE Bot SHALL fail to start with a clear error message

### Requirement 9: Graceful Shutdown

**User Story:** As a system administrator, I want the bot to shut down gracefully when terminated, so that no data is lost or corrupted.

#### Acceptance Criteria

1. WHEN the Bot receives a shutdown signal (SIGTERM, SIGINT), THE Bot SHALL stop accepting new commands
2. WHEN shutting down, THE Bot SHALL complete any in-progress Database operations
3. WHEN shutting down, THE Bot SHALL close all Database connections properly
4. WHEN shutdown is complete, THE Bot SHALL log a shutdown confirmation message

### Requirement 10: Testing Infrastructure

**User Story:** As a developer, I want comprehensive unit and integration tests, so that I can verify correctness and prevent regressions.

#### Acceptance Criteria

1. THE test suite SHALL include unit tests for all database operations
2. THE test suite SHALL include unit tests for date calculation and formatting logic
3. THE test suite SHALL include unit tests for expense validation and limit checking
4. THE test suite SHALL include unit tests for command parsing
5. THE test suite SHALL include integration tests for bot command handlers
6. THE test suite SHALL use property-based testing for date calculations and numeric operations
7. WHEN tests require a Database, THE test suite SHALL use a test database or mocking strategy
8. THE test suite SHALL achieve at least 80% code coverage for business logic
