// Unit tests for bot command handlers
// Tests for task 10.1

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::db::repository::mock::MockRepository;
    use crate::db::repository::RepositoryTrait;
    use crate::services::expense_service::ExpenseService;
    use crate::services::user_service::UserService;
    use rust_decimal_macros::dec;
    use std::sync::Arc;

    // Note: These tests verify the business logic of the handlers.
    // We cannot easily test the actual Telegram bot interactions without mocking
    // the entire teloxide framework, so we focus on testing the error formatting
    // and the logic that calls the services.

    #[test]
    fn test_format_error_message_database() {
        let error = BotError::Database(sqlx::Error::Protocol("test".to_string()));
        let msg = format_error_message(&error);
        assert!(msg.contains("Unable to process your request"));
        assert!(!msg.contains("Protocol")); // Should not expose internal details
        assert!(!msg.contains("sqlx")); // Should not expose internal details
    }

    #[test]
    fn test_format_error_message_config() {
        let error = BotError::Config("Missing token".to_string());
        let msg = format_error_message(&error);
        assert!(msg.contains("Configuration error"));
        assert!(msg.contains("Missing token"));
    }

    #[test]
    fn test_format_error_message_invalid_input() {
        let error = BotError::InvalidInput("Not a number".to_string());
        let msg = format_error_message(&error);
        assert!(msg.contains("Invalid input"));
        assert!(msg.contains("Not a number"));
    }

    #[test]
    fn test_format_error_message_user_not_found() {
        let error = BotError::UserNotFound("alice".to_string());
        let msg = format_error_message(&error);
        assert!(msg.contains("register first"));
        assert!(msg.contains("/start"));
        assert!(!msg.contains("alice")); // Should not expose username in error
    }

    #[test]
    fn test_format_error_message_parse() {
        let error = BotError::Parse("Invalid decimal".to_string());
        let msg = format_error_message(&error);
        assert!(msg.contains("Parse error"));
        assert!(msg.contains("Invalid decimal"));
    }

    #[test]
    fn test_format_error_messages_are_user_friendly() {
        // Verify that all error messages:
        // 1. Don't contain stack traces
        // 2. Don't contain technical jargon like "sqlx", "Protocol", etc.
        // 3. Provide actionable guidance where appropriate

        let errors = vec![
            BotError::Database(sqlx::Error::Protocol("test".to_string())),
            BotError::Config("test".to_string()),
            BotError::InvalidInput("test".to_string()),
            BotError::UserNotFound("test".to_string()),
            BotError::Parse("test".to_string()),
        ];

        for error in errors {
            let msg = format_error_message(&error);

            // Should not contain technical terms
            assert!(!msg.contains("sqlx"));
            assert!(!msg.contains("Protocol"));
            assert!(!msg.contains("Error::"));
            assert!(!msg.contains("panic"));

            // Should be non-empty
            assert!(!msg.is_empty());

            // Should contain emoji or clear formatting
            assert!(
                msg.contains("⚠️")
                    || msg.contains("❌")
                    || msg.contains("error")
                    || msg.contains("Error")
            );
        }
    }

    // Integration-style tests that verify the handler logic works with services

    #[tokio::test]
    async fn test_user_registration_flow() {
        let repo = Arc::new(MockRepository::new()) as Arc<dyn RepositoryTrait>;
        let user_service = Arc::new(UserService::new(repo.clone(), dec!(210.00)));

        // Register a new user
        let result = user_service.register_user("alice".to_string(), 12345).await;
        assert!(result.is_ok());

        // Verify the user was created
        let config = repo.get_user_config("alice").await.unwrap().unwrap();
        assert_eq!(config.username, "alice");
        assert_eq!(config.chat_id, 12345);
        assert_eq!(config.pay_limit, dec!(210.00));
    }

    #[tokio::test]
    async fn test_expense_addition_flow() {
        let repo = Arc::new(MockRepository::new()) as Arc<dyn RepositoryTrait>;
        let user_service = Arc::new(UserService::new(repo.clone(), dec!(210.00)));
        let expense_service = Arc::new(ExpenseService::new(repo.clone()));

        // Register a user first
        user_service
            .register_user("bob".to_string(), 67890)
            .await
            .unwrap();

        // Add an expense
        let result = expense_service.add_expense("bob", dec!(45.50)).await;
        assert!(result.is_ok());

        match result.unwrap() {
            AddExpenseResult::Success {
                new_total,
                remaining,
            } => {
                assert_eq!(new_total, dec!(45.50));
                assert_eq!(remaining, dec!(164.50));
            }
            _ => panic!("Expected Success result"),
        }
    }

    #[tokio::test]
    async fn test_monthly_summary_flow() {
        let repo = Arc::new(MockRepository::new()) as Arc<dyn RepositoryTrait>;
        let user_service = Arc::new(UserService::new(repo.clone(), dec!(210.00)));
        let expense_service = Arc::new(ExpenseService::new(repo.clone()));

        // Register a user and add some expenses
        user_service
            .register_user("charlie".to_string(), 11111)
            .await
            .unwrap();
        expense_service
            .add_expense("charlie", dec!(50.00))
            .await
            .unwrap();
        expense_service
            .add_expense("charlie", dec!(30.00))
            .await
            .unwrap();

        // Get the summary
        let result = expense_service.get_monthly_summary("charlie").await;
        assert!(result.is_ok());

        let summary = result.unwrap();
        assert_eq!(summary.total_spent, dec!(80.00));
        assert_eq!(summary.limit, dec!(210.00));
        assert_eq!(summary.remaining, dec!(130.00));
    }

    #[tokio::test]
    async fn test_limit_update_flow() {
        let repo = Arc::new(MockRepository::new()) as Arc<dyn RepositoryTrait>;
        let user_service = Arc::new(UserService::new(repo.clone(), dec!(210.00)));

        // Register a user
        user_service
            .register_user("dave".to_string(), 22222)
            .await
            .unwrap();

        // Update the limit
        let result = user_service.update_limit("dave", dec!(300.00)).await;
        assert!(result.is_ok());

        // Verify the limit was updated
        let config = repo.get_user_config("dave").await.unwrap().unwrap();
        assert_eq!(config.pay_limit, dec!(300.00));
    }

    #[tokio::test]
    async fn test_limit_exceeded_flow() {
        let repo = Arc::new(MockRepository::new()) as Arc<dyn RepositoryTrait>;
        let user_service = Arc::new(UserService::new(repo.clone(), dec!(100.00)));
        let expense_service = Arc::new(ExpenseService::new(repo.clone()));

        // Register a user with a low limit
        user_service
            .register_user("eve".to_string(), 33333)
            .await
            .unwrap();

        // Try to add an expense that exceeds the limit
        let result = expense_service.add_expense("eve", dec!(150.00)).await;
        assert!(result.is_ok());

        match result.unwrap() {
            AddExpenseResult::LimitExceeded {
                current,
                attempted,
                limit,
            } => {
                assert_eq!(current, dec!(0.00));
                assert_eq!(attempted, dec!(150.00));
                assert_eq!(limit, dec!(100.00));
            }
            _ => panic!("Expected LimitExceeded result"),
        }
    }

    #[tokio::test]
    async fn test_invalid_limit_update() {
        let repo = Arc::new(MockRepository::new()) as Arc<dyn RepositoryTrait>;
        let user_service = Arc::new(UserService::new(repo.clone(), dec!(210.00)));

        // Register a user
        user_service
            .register_user("frank".to_string(), 44444)
            .await
            .unwrap();

        // Try to update with negative limit
        let result = user_service.update_limit("frank", dec!(-50.00)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BotError::InvalidInput(msg) => {
                assert!(msg.contains("positive"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_user_not_found_flow() {
        let repo = Arc::new(MockRepository::new()) as Arc<dyn RepositoryTrait>;
        let expense_service = Arc::new(ExpenseService::new(repo.clone()));

        // Try to add an expense for a non-existent user
        let result = expense_service.add_expense("ghost", dec!(50.00)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BotError::UserNotFound(username) => {
                assert_eq!(username, "ghost");
            }
            _ => panic!("Expected UserNotFound error"),
        }
    }
}
