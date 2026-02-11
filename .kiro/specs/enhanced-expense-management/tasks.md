# Implementation Plan: Enhanced Expense Management

## Overview

This implementation plan breaks down the enhanced expense management feature into discrete coding tasks. The approach follows a bottom-up strategy: database layer first, then service layer, then bot handlers, and finally startup notifications. Each task builds on previous work and includes testing sub-tasks to validate functionality incrementally.

## Tasks

- [~] 1. Extend repository layer with new database operations
  - [ ] 1.1 Add get_current_month_expenses method to RepositoryTrait and Repository
    - Implement SQL query with current month filtering and chronological ordering
    - Add corresponding method to MockRepository for testing
    - _Requirements: 1.1, 1.5_
  
  - [ ] 1.2 Write property test for current month expense retrieval
    - **Property 1: Current Month Expense Retrieval Completeness**
    - **Validates: Requirements 1.1**
  
  - [ ] 1.3 Add delete_current_month_expenses method to RepositoryTrait and Repository
    - Implement SQL DELETE with current month filtering
    - Return count of deleted rows
    - Add corresponding method to MockRepository
    - _Requirements: 3.1, 3.2_
  
  - [ ] 1.4 Write property test for delete current month expenses
    - **Property 8: Clear Current Month Completeness**
    - **Property 9: Previous Month Protection During Clear**
    - **Validates: Requirements 3.1, 3.2, 3.3**
  
  - [ ] 1.5 Add delete_last_current_month_expense method to RepositoryTrait and Repository
    - Implement query to find most recent current month expense
    - Delete the identified expense and return its details
    - Handle same-day expenses with ID tiebreaker
    - Add corresponding method to MockRepository
    - _Requirements: 4.1, 4.2, 4.4_
  
  - [ ] 1.6 Write property test for delete last expense
    - **Property 10: Last Expense Identification**
    - **Property 11: Remove Last Scope Restriction**
    - **Validates: Requirements 4.1, 4.2, 4.4, 5.3**
  
  - [ ] 1.7 Add get_year_summary method to RepositoryTrait and Repository
    - Implement SQL query with GROUP BY month and SUM aggregation
    - Return vector of (month, total) tuples
    - Add corresponding method to MockRepository
    - _Requirements: 2.1, 2.4_
  
  - [ ] 1.8 Write property test for year summary
    - **Property 4: Year Summary Completeness**
    - **Property 6: Year Summary Chronological Ordering**
    - **Validates: Requirements 2.1, 2.4**
  
  - [ ] 1.9 Add get_all_chat_ids method to RepositoryTrait and Repository
    - Implement SQL query to get distinct chat IDs from config table
    - Add corresponding method to MockRepository
    - _Requirements: 6.1_
  
  - [ ] 1.10 Write property test for chat ID retrieval
    - **Property 12: Chat ID Retrieval for Notifications**
    - **Validates: Requirements 6.1**

- [ ] 2. Extend ExpenseService with new business logic methods
  - [ ] 2.1 Add ExpenseDetail and YearSummary data structures
    - Define ExpenseDetail struct with day, amount, and date fields
    - Define YearSummary struct with year, monthly_totals, and grand_total
    - Define MonthTotal struct with month, month_name, and total
    - _Requirements: 1.2, 1.3, 2.2, 2.5_
  
  - [ ] 2.2 Implement list_current_month_expenses method
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
  
  - [ ] 2.5 Implement clear_current_month method
    - Call repository delete_current_month_expenses
    - Return count of deleted expenses
    - Handle empty month case
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  
  - [ ]* 2.6 Write unit test for empty month clear
    - Test that clearing empty month returns zero count
    - _Requirements: 3.4_
  
  - [ ] 2.7 Implement remove_last_expense method
    - Call repository delete_last_current_month_expense
    - Transform result to ExpenseDetail if expense existed
    - Handle empty month case
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 5.3_
  
  - [ ]* 2.8 Write unit test for empty month remove last
    - Test that removing from empty month returns None
    - _Requirements: 4.3_
  
  - [ ] 2.9 Implement get_year_summary method
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

- [ ] 3. Create VersionService for startup notifications
  - [ ] 3.1 Create version_service.rs module
    - Define VersionService struct with repository dependency
    - Implement new() constructor
    - Add module declaration in services/mod.rs
    - _Requirements: 6.1, 6.3, 6.4_
  
  - [ ] 3.2 Implement get_notification_targets method
    - Call repository get_all_chat_ids
    - Return list of chat IDs
    - _Requirements: 6.1_
  
  - [ ] 3.3 Implement get_current_version method
    - Use env!("CARGO_PKG_VERSION") to get version at compile time
    - Return as static string
    - _Requirements: 6.3_
  
  - [ ] 3.4 Implement get_change_description method
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

- [ ] 4. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 5. Add bot command handlers for new operations
  - [ ] 5.1 Implement handle_list_month command handler
    - Parse command and extract username from message
    - Call expense_service.list_current_month_expenses
    - Format response with day and amount for each expense
    - Handle empty month case with appropriate message
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 5.2 Implement handle_year_summary command handler
    - Parse command and extract username from message
    - Call expense_service.get_year_summary
    - Format response with month names, totals, and grand total
    - Handle empty year case
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  
  - [ ] 5.3 Implement handle_clear_month command handler
    - Parse command and extract username from message
    - Call expense_service.clear_current_month
    - Format confirmation message with count of deleted expenses
    - Handle empty month case
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  
  - [ ] 5.4 Implement handle_remove_last command handler
    - Parse command and extract username from message
    - Call expense_service.remove_last_expense
    - Format confirmation message with deleted expense details
    - Handle empty month case
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 5.3_
  
  - [ ] 5.5 Register new command handlers in dispatcher
    - Add /list_month command to dispatcher
    - Add /year_summary command to dispatcher
    - Add /clear_month command to dispatcher
    - Add /remove_last command to dispatcher
    - Update command descriptions for /help
    - _Requirements: 1.1, 2.1, 3.1, 4.1_

- [ ] 6. Implement startup notification system
  - [ ] 6.1 Create send_startup_notifications function in main.rs
    - Initialize VersionService with repository
    - Get current version and change description
    - Get all chat IDs from version service
    - Format notification message with version and changes
    - Send message to each chat ID
    - Log any send failures but continue with remaining chats
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
  
  - [ ] 6.2 Integrate startup notifications into main function
    - Call send_startup_notifications after bot initialization
    - Call before starting dispatcher
    - Handle errors gracefully (log but don't prevent bot startup)
    - _Requirements: 6.1, 6.2_
  
  - [ ]* 6.3 Write unit test for notification message formatting
    - Test that message includes version and changelog
    - Test default message when metadata missing
    - _Requirements: 6.3, 6.4_

- [ ] 7. Update Cargo.toml with changelog metadata
  - [ ] 7.1 Add package.metadata.changelog section to Cargo.toml
    - Add description field with current feature changes
    - Document the metadata format for future updates
    - _Requirements: 6.4_

- [ ] 8. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- The implementation follows the existing architecture patterns in the codebase
