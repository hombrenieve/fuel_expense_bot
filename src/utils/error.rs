// Error handling module
// Will be implemented in task 2.1

use thiserror::Error;

#[derive(Debug, Error)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let error = BotError::Config("Missing token".to_string());
        assert_eq!(error.to_string(), "Configuration error: Missing token");
    }

    #[test]
    fn test_invalid_input_error_display() {
        let error = BotError::InvalidInput("Not a number".to_string());
        assert_eq!(error.to_string(), "Invalid input: Not a number");
    }

    #[test]
    fn test_user_not_found_error_display() {
        let error = BotError::UserNotFound("alice".to_string());
        assert_eq!(error.to_string(), "User not found: alice");
    }

    #[test]
    fn test_parse_error_display() {
        let error = BotError::Parse("Invalid decimal format".to_string());
        assert_eq!(error.to_string(), "Parse error: Invalid decimal format");
    }

    #[test]
    fn test_error_is_send_and_sync() {
        // Verify that BotError implements Send and Sync for async contexts
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<BotError>();
        assert_sync::<BotError>();
    }

    #[test]
    fn test_result_type_alias() {
        // Verify that Result type alias works correctly
        let ok_result: Result<i32> = Ok(42);
        assert!(ok_result.is_ok());
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: Result<i32> = Err(BotError::Config("test".to_string()));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_sqlx_error_conversion() {
        // Test that sqlx::Error can be converted to BotError
        // We'll create a simple sqlx error by using a protocol error
        let sqlx_err = sqlx::Error::Protocol("test protocol error".to_string());
        let bot_error: BotError = sqlx_err.into();

        match bot_error {
            BotError::Database(_) => {
                // Success - the error was converted correctly
                assert!(bot_error.to_string().contains("Database error"));
            }
            _ => panic!("Expected Database variant"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        // Verify that Debug trait works (derived automatically)
        let error = BotError::Config("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_all_error_variants() {
        // Ensure all error variants can be created and displayed
        let errors = vec![
            BotError::Config("config error".to_string()),
            BotError::InvalidInput("invalid input".to_string()),
            BotError::UserNotFound("user123".to_string()),
            BotError::Parse("parse error".to_string()),
        ];

        for error in errors {
            // Each error should have a non-empty display string
            assert!(!error.to_string().is_empty());
        }
    }
}
