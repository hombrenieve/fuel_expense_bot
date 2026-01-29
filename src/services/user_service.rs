// User service module
// Implements task 7.1

use rust_decimal::Decimal;
use std::sync::Arc;

use crate::db::models::UserConfig;
use crate::db::repository::RepositoryTrait;
use crate::utils::error::{BotError, Result};

/// Result of a user registration attempt
#[derive(Debug, Clone, PartialEq)]
pub enum RegistrationResult {
    /// A new user was successfully created
    NewUser,
    /// The user was already registered
    AlreadyRegistered,
}

/// Service for managing user-related operations
///
/// This service handles user registration, configuration updates, and retrieval.
/// It wraps the repository layer and provides business logic for user management.
///
/// # Requirements
/// - Validates: Requirements 1.1, 1.2, 1.3, 4.1, 4.2, 4.3
pub struct UserService {
    repo: Arc<dyn RepositoryTrait>,
    default_limit: Decimal,
}

impl UserService {
    /// Create a new UserService
    ///
    /// # Arguments
    /// * `repo` - The repository implementation to use for database operations
    /// * `default_limit` - The default monthly spending limit for new users
    pub fn new(repo: Arc<dyn RepositoryTrait>, default_limit: Decimal) -> Self {
        Self {
            repo,
            default_limit,
        }
    }

    /// Register a new user or acknowledge existing registration
    ///
    /// This function attempts to create a new user with the provided username and chat ID.
    /// If the user already exists, it returns AlreadyRegistered instead of an error.
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `chat_id` - The Telegram chat ID
    ///
    /// # Returns
    /// * `Ok(RegistrationResult::NewUser)` if a new user was created
    /// * `Ok(RegistrationResult::AlreadyRegistered)` if the user already exists
    /// * `Err(BotError)` if a database error occurs (other than duplicate key)
    ///
    /// # Requirements
    /// - Validates: Requirements 1.1, 1.2, 1.3
    pub async fn register_user(
        &self,
        username: String,
        chat_id: i64,
    ) -> Result<RegistrationResult> {
        match self
            .repo
            .create_user(&username, chat_id, self.default_limit)
            .await
        {
            Ok(()) => Ok(RegistrationResult::NewUser),
            Err(BotError::Database(e)) => {
                // Check if this is a duplicate key error
                let error_msg = e.to_string();
                if error_msg.contains("Duplicate") || error_msg.contains("duplicate") {
                    Ok(RegistrationResult::AlreadyRegistered)
                } else {
                    Err(BotError::Database(e))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Update a user's monthly spending limit
    ///
    /// This function validates the new limit and updates it in the database.
    /// The limit must be a positive number greater than zero.
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    /// * `new_limit` - The new monthly spending limit
    ///
    /// # Returns
    /// * `Ok(())` if the limit was updated successfully
    /// * `Err(BotError::InvalidInput)` if the limit is invalid (negative or zero)
    /// * `Err(BotError::UserNotFound)` if the user doesn't exist
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 4.1, 4.2, 4.3
    pub async fn update_limit(&self, username: &str, new_limit: Decimal) -> Result<()> {
        // Validate that the limit is positive
        if new_limit <= Decimal::ZERO {
            return Err(BotError::InvalidInput(format!(
                "Limit must be a positive number, got: {}",
                new_limit
            )));
        }

        // Update the limit in the database
        self.repo.update_user_limit(username, new_limit).await
    }

    /// Get a user's configuration
    ///
    /// # Arguments
    /// * `username` - The Telegram username
    ///
    /// # Returns
    /// * `Ok(UserConfig)` if the user exists
    /// * `Err(BotError::UserNotFound)` if the user doesn't exist
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirements 1.1, 4.1
    pub async fn get_config(&self, username: &str) -> Result<UserConfig> {
        match self.repo.get_user_config(username).await? {
            Some(config) => Ok(config),
            None => Err(BotError::UserNotFound(username.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::mock::MockRepository;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_register_new_user() {
        let repo = Arc::new(MockRepository::new());
        let service = UserService::new(repo.clone(), dec!(210.00));

        let result = service.register_user("alice".to_string(), 12345).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RegistrationResult::NewUser);

        // Verify the user was created with correct values
        let config = repo.get_user_config("alice").await.unwrap().unwrap();
        assert_eq!(config.username, "alice");
        assert_eq!(config.chat_id, 12345);
        assert_eq!(config.pay_limit, dec!(210.00));
    }

    #[tokio::test]
    async fn test_register_existing_user() {
        let repo = Arc::new(MockRepository::new());
        let service = UserService::new(repo.clone(), dec!(210.00));

        // Register user first time
        let result1 = service.register_user("bob".to_string(), 67890).await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), RegistrationResult::NewUser);

        // Register same user again
        let result2 = service.register_user("bob".to_string(), 67890).await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), RegistrationResult::AlreadyRegistered);
    }

    #[tokio::test]
    async fn test_update_limit_valid() {
        let repo = Arc::new(MockRepository::new());
        let service = UserService::new(repo.clone(), dec!(210.00));

        // Register a user first
        service
            .register_user("charlie".to_string(), 11111)
            .await
            .unwrap();

        // Update the limit
        let result = service.update_limit("charlie", dec!(300.00)).await;
        assert!(result.is_ok());

        // Verify the limit was updated
        let config = repo.get_user_config("charlie").await.unwrap().unwrap();
        assert_eq!(config.pay_limit, dec!(300.00));
    }

    #[tokio::test]
    async fn test_update_limit_negative() {
        let repo = Arc::new(MockRepository::new());
        let service = UserService::new(repo.clone(), dec!(210.00));

        // Register a user first
        service
            .register_user("dave".to_string(), 22222)
            .await
            .unwrap();

        // Try to update with negative limit
        let result = service.update_limit("dave", dec!(-50.00)).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::InvalidInput(msg) => {
                assert!(msg.contains("positive"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_update_limit_zero() {
        let repo = Arc::new(MockRepository::new());
        let service = UserService::new(repo.clone(), dec!(210.00));

        // Register a user first
        service
            .register_user("eve".to_string(), 33333)
            .await
            .unwrap();

        // Try to update with zero limit
        let result = service.update_limit("eve", Decimal::ZERO).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::InvalidInput(msg) => {
                assert!(msg.contains("positive"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_update_limit_nonexistent_user() {
        let repo = Arc::new(MockRepository::new());
        let service = UserService::new(repo, dec!(210.00));

        // Try to update limit for a user that doesn't exist
        let result = service.update_limit("ghost", dec!(100.00)).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::UserNotFound(username) => {
                assert_eq!(username, "ghost");
            }
            _ => panic!("Expected UserNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_config_existing_user() {
        let repo = Arc::new(MockRepository::new());
        let service = UserService::new(repo.clone(), dec!(210.00));

        // Register a user
        service
            .register_user("frank".to_string(), 44444)
            .await
            .unwrap();

        // Get the config
        let result = service.get_config("frank").await;
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.username, "frank");
        assert_eq!(config.chat_id, 44444);
        assert_eq!(config.pay_limit, dec!(210.00));
    }

    #[tokio::test]
    async fn test_get_config_nonexistent_user() {
        let repo = Arc::new(MockRepository::new());
        let service = UserService::new(repo, dec!(210.00));

        // Try to get config for a user that doesn't exist
        let result = service.get_config("phantom").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BotError::UserNotFound(username) => {
                assert_eq!(username, "phantom");
            }
            _ => panic!("Expected UserNotFound error"),
        }
    }
}
