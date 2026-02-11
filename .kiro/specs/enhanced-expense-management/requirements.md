# Requirements Document

## Introduction

This document specifies requirements for enhanced expense management capabilities and version notification features in the Telegram fuel bot. The enhancements provide users with better control over their expense data through detailed listing, selective deletion, and protection mechanisms. Additionally, the system will notify users when new versions are deployed.

## Glossary

- **Bot**: The Telegram fuel bot system that manages expense tracking
- **User**: A person interacting with the Bot through Telegram
- **Expense**: A recorded financial transaction with an amount and date
- **Current_Month**: The calendar month matching the current system date
- **Previous_Month**: Any calendar month before the Current_Month
- **Chat**: A Telegram conversation context where the Bot operates
- **Version**: A specific release of the Bot software identified by semantic versioning
- **Change_Description**: Text describing modifications in a new Version, stored in Cargo.toml metadata

## Requirements

### Requirement 1: Detailed Expense Listing

**User Story:** As a user, I want to view all my current month's expenses with the exact day each was recorded, so that I can review my spending patterns and verify entries.

#### Acceptance Criteria

1. WHEN a user requests the current month's expense list, THE Bot SHALL return all expenses recorded in the Current_Month
2. WHEN displaying each expense, THE Bot SHALL include the day of the month when the expense was recorded
3. WHEN displaying each expense, THE Bot SHALL include the expense amount
4. WHEN the Current_Month has no expenses, THE Bot SHALL return a message indicating no expenses exist
5. THE Bot SHALL order expenses chronologically by recording date

### Requirement 2: Current Year Summary

**User Story:** As a user, I want to view a summary of the current year's expenses by month, so that I can understand my spending trends over the year.

#### Acceptance Criteria

1. WHEN a user requests the current year summary, THE Bot SHALL return expense totals for each month in the current calendar year
2. WHEN displaying monthly totals, THE Bot SHALL include the month name and total amount
3. WHEN a month has no expenses, THE Bot SHALL display zero or omit that month from the summary
4. THE Bot SHALL order the summary chronologically from January to December
5. WHEN displaying the summary, THE Bot SHALL include a grand total for the entire year

### Requirement 3: Clear Current Month Expenses

**User Story:** As a user, I want to remove all expenses from the current month, so that I can start fresh if I made mistakes or want to re-enter data.

#### Acceptance Criteria

1. WHEN a user requests to clear current month expenses, THE Bot SHALL remove all expenses from the Current_Month
2. WHEN clearing current month expenses, THE Bot SHALL preserve all expenses from Previous_Month periods
3. WHEN the clear operation completes, THE Bot SHALL confirm the number of expenses removed
4. WHEN the Current_Month has no expenses, THE Bot SHALL inform the user that no expenses were removed

### Requirement 4: Remove Last Expense

**User Story:** As a user, I want to remove the most recent expense from the current month, so that I can quickly correct mistakes without clearing all data.

#### Acceptance Criteria

1. WHEN a user requests to remove the last expense, THE Bot SHALL identify the most recent expense in the Current_Month
2. WHEN a most recent expense exists, THE Bot SHALL remove that expense and confirm the removal with expense details
3. WHEN the Current_Month has no expenses, THE Bot SHALL inform the user that no expense was removed
4. WHEN multiple expenses exist on the same day, THE Bot SHALL remove the one with the latest timestamp

### Requirement 5: Previous Month Protection

**User Story:** As a user, I want my previous months' expenses to be protected from modification, so that my historical records remain accurate and auditable.

#### Acceptance Criteria

1. THE Bot SHALL prevent removal of any expense from Previous_Month periods
2. WHEN executing clear current month operation, THE Bot SHALL only affect Current_Month expenses
3. WHEN executing remove last expense operation, THE Bot SHALL only consider Current_Month expenses
4. THE Bot SHALL maintain data integrity for all Previous_Month expenses during any deletion operation

### Requirement 6: Startup Notifications

**User Story:** As a user, I want to be notified when the bot starts, so that I know the service is running and can see the current version and features.

#### Acceptance Criteria

1. WHEN the Bot starts, THE Bot SHALL identify all active Chat contexts
2. WHEN the Bot starts, THE Bot SHALL send a notification message to each active Chat
3. WHEN sending startup notifications, THE Bot SHALL include the current Version number
4. WHEN sending startup notifications, THE Bot SHALL include the Change_Description from Cargo.toml metadata
