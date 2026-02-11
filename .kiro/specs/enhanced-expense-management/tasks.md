# Implementation Plan: Enhanced Expense Management

## Overview

This implementation plan breaks down the enhanced expense management feature into discrete coding tasks. The approach follows a bottom-up strategy: database layer first, then service layer, then bot handlers, and finally startup notifications. Each task builds on previous work and includes testing sub-tasks to validate functionality incrementally.

## Tasks

- [x] 1. Extend repository layer with new database operations
  - [x] 1.1 Add get_current_month_expenses method to RepositoryTrait and Repository
  - [x] 1.2 Write property test for current month expense retrieval
  - [x] 1.3 Add delete_current_month_expenses method to RepositoryTrait and Repository
  - [x] 1.4 Write property test for delete current month expenses
  - [x] 1.5 Add delete_last_current_month_expense method to RepositoryTrait and Repository
  - [x] 1.6 Write property test for delete last expense
  - [x] 1.7 Add get_year_summary method to RepositoryTrait and Repository
  - [x] 1.8 Write property test for year summary
  - [x] 1.9 Add get_all_chat_ids method to RepositoryTrait and Repository
  - [x] 1.10 Write property test for chat ID retrieval

- [x] 2. Extend ExpenseService with new business logic methods
  - [x] 2.1 Add ExpenseDetail and YearSummary data structures
    - Define ExpenseDetail struct with day, amount, and date fields
    - Define YearSummary struct with year, monthly_totals, and grand_total
    - Define MonthTotal struct with month, month_name, and total
    - _Requirements: 1.2, 1.3, 2.2, 2.5_
  
  - [x] 2.2 Implement list_current_month_expenses method
    - Call repository get_current_month_expenses
    - Transform Expense models to ExpenseDetail with day extraction
    - Handle empty month case
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ]* 2.3 Write property test for expense listing
    - **Property 2: Expense Display Completeness**
    - **Property 3: Chronological Ordering of Expenses**
    - **Validates: Requirements 1.2, 1.3, 1.5**
  
  - [ ]* 2.4 Write unit test for empty month listing
    - Test that empty month returns appropriate message
    - _Requirements: 1.4_
  
  - [x] 2.5 Implement clear_current_month method
    - Call repository delete_current_month_expenses
    - Return count of deleted expenses
    - Handle empty month case
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  
  - [ ]* 2.6 Write unit test for empty month clear
    - Test that clearing empty month returns zero count
    - _Requirements: 3.4_
  
  - [x] 2.7 Implement remove_last_expense method
    - Call repository delete_last_current_month_expense
    - Transform result to ExpenseDetail if expense existed
    - Handle empty month case
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 5.3_
  
  - [ ]* 2.8 Write unit test for empty month remove last
    - Test that removing from empty month returns None
    - _Requirements: 4.3_
  
  - [x] 2.9 Implement get_year_summary method
    - Call repository get_year_summary
    - Transform month numbers to month names
    - Calculate grand total from monthly totals
    - Handle empty year case
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  
  - [ ]* 2.10 Write property test for year summary totals
    - **Property 5: Monthly Total Display Format**
    - **Property 7: Year Summary Grand Total Accuracy**
    - **Validates: Requirements 2.2, 2.5**
  
  - [ ]* 2.11 Write unit test for empty year summary
    - Test that year with no expenses returns empty summary with zero total
    - _Requirements: 2.3_

- [x] 3. Create VersionService for startup notifications
  - [x] 3.1 Create version_service.rs module
    - Define VersionService struct with repository dependency
    - Implement new() constructor
    - Add module declaration in services/mod.rs
    - _Requirements: 6.1, 6.3, 6.4_
  
  - [x] 3.2 Implement get_notification_targets method
    - Call repository get_all_chat_ids
    - Return list of chat IDs
    - _Requirements: 6.1_
  
  - [x] 3.3 Implement get_current_version method
    - Use env!("CARGO_PKG_VERSION") to get version at compile time
    - Return as static string
    - _Requirements: 6.3_
  
  - [x] 3.4 Implement get_change_description method
    - Read from Cargo.toml package.metadata.changelog.description
    - Use compile-time environment variable or include_str!
    - Provide default message if metadata missing
    - _Requirements: 6.4_
  
  - [ ]* 3.5 Write property test for notification message completeness
    - **Property 13: Startup Notification Message Completeness**
    - **Validates: Requirements 6.3, 6.4**
  
  - [ ]* 3.6 Write unit test for missing metadata handling
    - Test that missing changelog metadata uses default message
    - _Requirements: 6.4_

- [x] 4. Add bot command handlers for new operations
  - [x] 4.1 Implement handle_list_month command handler
    - Parse command and extract username from message
    - Call expense_service.list_current_month_expenses
    - Format response with day and amount for each expense
    - Handle empty month case with appropriate message
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 4.2 Implement handle_year_summary command handler
    - Parse command and extract username from message
    - Call expense_service.get_year_summary
    - Format response with month names, totals, and grand total
    - Handle empty year case
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  
  - [x] 4.3 Implement handle_clear_month command handler
    - Parse command and extract username from message
    - Call expense_service.clear_current_month
    - Format confirmation message with count of deleted expenses
    - Handle empty month case
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  
  - [x] 4.4 Implement handle_remove_last command handler
    - Parse command and extract username from message
    - Call expense_service.remove_last_expense
    - Format confirmation message with deleted expense details
    - Handle empty month case
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 5.3_
  
  - [x] 4.5 Register new command handlers in dispatcher
    - Add /list_month command to dispatcher
    - Add /year_summary command to dispatcher
    - Add /clear_month command to dispatcher
    - Add /remove_last command to dispatcher
    - Update command descriptions for /help
    - _Requirements: 1.1, 2.1, 3.1, 4.1_

- [ ] 5. Implement startup notification system
  - [ ] 5.1 Create send_startup_notifications function in main.rs
    - Initialize VersionService with repository
    - Get current version and change description
    - Get all chat IDs from version service
    - Format notification message with version and changes
    - Send message to each chat ID
    - Log any send failures but continue with remaining chats
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
  
  - [ ] 5.2 Integrate startup notifications into main function
    - Call send_startup_notifications after bot initialization
    - Call before starting dispatcher
    - Handle errors gracefully (log but don't prevent bot startup)
    - _Requirements: 6.1, 6.2_
  
  - [ ]* 5.3 Write unit test for notification message formatting
    - Test that message includes version and changelog
    - Test default message when metadata missing
    - _Requirements: 6.3, 6.4_

- [ ] 6. Update Cargo.toml with changelog metadata
  - [ ] 6.1 Add package.metadata.changelog section to Cargo.toml
    - Add description field with current feature changes
    - Document the metadata format for future updates
    - _Requirements: 6.4_

- [ ] 7. Final validation
  - [ ] 7.1 Run all tests to ensure everything passes
  - [ ] 7.2 Manually test all new commands in Telegram
  - [ ] 7.3 Verify startup notifications are sent correctly

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- The implementation follows the existing architecture patterns in the codebase
