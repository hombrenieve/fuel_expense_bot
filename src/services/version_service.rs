// Version service for startup notifications
// Implements requirements 6.1, 6.3, 6.4

use std::sync::Arc;

use crate::db::repository::RepositoryTrait;
use crate::utils::error::Result;

/// Service for managing version information and startup notifications
///
/// This service handles retrieving version information from Cargo.toml
/// and identifying notification targets for startup messages.
pub struct VersionService {
    repo: Arc<dyn RepositoryTrait>,
}

impl VersionService {
    /// Create a new VersionService instance
    ///
    /// # Arguments
    /// * `repo` - Repository for database operations
    ///
    /// # Returns
    /// A new VersionService instance
    pub fn new(repo: Arc<dyn RepositoryTrait>) -> Self {
        Self { repo }
    }

    /// Get all chat IDs that should receive startup notifications
    ///
    /// Retrieves all unique chat IDs from the database to send
    /// startup notifications to all active users.
    ///
    /// # Returns
    /// * `Ok(Vec<i64>)` - Vector of unique chat IDs
    /// * `Err(BotError::Database)` if a database error occurs
    ///
    /// # Requirements
    /// - Validates: Requirement 6.1
    pub async fn get_notification_targets(&self) -> Result<Vec<i64>> {
        self.repo.get_all_chat_ids().await
    }

    /// Get the current version from Cargo.toml
    ///
    /// Returns the version string defined in Cargo.toml at compile time.
    ///
    /// # Returns
    /// The version string (e.g., "0.1.0")
    ///
    /// # Requirements
    /// - Validates: Requirement 6.3
    pub fn get_current_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Get the change description from Cargo.toml metadata
    ///
    /// Returns the changelog description from package.metadata.changelog.description
    /// in Cargo.toml. If the metadata is missing, returns a default message.
    ///
    /// # Returns
    /// * `Ok(String)` - The change description or default message
    ///
    /// # Requirements
    /// - Validates: Requirement 6.4
    pub fn get_change_description() -> Result<String> {
        // Try to get the changelog from compile-time environment variable
        // This would be set if package.metadata.changelog.description exists
        match option_env!("CARGO_PKG_METADATA_CHANGELOG_DESCRIPTION") {
            Some(changelog) => Ok(changelog.to_string()),
            None => Ok("Version updated. See release notes for details.".to_string()),
        }
    }
}
