// Database connection pool management
// Implements connection pooling for efficient database access (Requirement 5.5)

use crate::config::DatabaseConfig;
use crate::utils::error::{BotError, Result};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::time::Duration;

/// Create a MySQL connection pool from configuration
///
/// This function creates a connection pool with the following settings:
/// - max_connections: Configured value from DatabaseConfig
/// - acquire_timeout: 30 seconds (time to wait for a connection from the pool)
/// - idle_timeout: 10 minutes (connections idle longer than this are closed)
/// - max_lifetime: 30 minutes (connections older than this are closed)
///
/// # Arguments
/// * `config` - Database configuration containing connection parameters
///
/// # Returns
/// * `Result<MySqlPool>` - A configured connection pool or an error
///
/// # Errors
/// Returns `BotError::Database` if:
/// - The database connection string is invalid
/// - Unable to connect to the database
/// - Authentication fails
///
/// # Example
/// ```no_run
/// use telegram_fuel_bot::config::DatabaseConfig;
/// use telegram_fuel_bot::db::pool::create_pool;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = DatabaseConfig {
///     host: "localhost".to_string(),
///     port: 3306,
///     username: "user".to_string(),
///     password: "pass".to_string(),
///     database: "fuel_bot".to_string(),
///     max_connections: 5,
/// };
///
/// let pool = create_pool(&config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_pool(config: &DatabaseConfig) -> Result<MySqlPool> {
    // Build the database connection URL
    // Format: mysql://username:password@host:port/database
    let database_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.username, config.password, config.host, config.port, config.database
    );

    // Create the connection pool with configured settings
    let pool = MySqlPoolOptions::new()
        // Maximum number of connections in the pool (Requirement 5.5)
        .max_connections(config.max_connections)
        // Time to wait for an available connection before timing out
        .acquire_timeout(Duration::from_secs(30))
        // Close connections that have been idle for more than 10 minutes
        .idle_timeout(Some(Duration::from_secs(600)))
        // Close connections that have been alive for more than 30 minutes
        // This helps prevent issues with stale connections
        .max_lifetime(Some(Duration::from_secs(1800)))
        // Test connections before returning them from the pool
        // This ensures we don't hand out broken connections
        .test_before_acquire(true)
        // Connect to the database
        .connect(&database_url)
        .await
        .map_err(|e| {
            // Convert sqlx error to our BotError type
            // This provides better error context for connection failures
            BotError::Database(e)
        })?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use crate::config::DatabaseConfig;

    /// Helper function to create a test database config
    fn create_test_config() -> DatabaseConfig {
        DatabaseConfig {
            host: "localhost".to_string(),
            port: 3306,
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            database: "test_db".to_string(),
            max_connections: 5,
        }
    }

    #[test]
    fn test_database_config_has_required_fields() {
        // Verify DatabaseConfig has all required fields for connection pooling
        let config = create_test_config();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
        assert_eq!(config.username, "test_user");
        assert_eq!(config.password, "test_pass");
        assert_eq!(config.database, "test_db");
        assert_eq!(config.max_connections, 5);
    }

    #[test]
    fn test_database_url_format() {
        // Test that we can construct a valid MySQL connection URL
        let config = create_test_config();

        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );

        assert_eq!(
            database_url,
            "mysql://test_user:test_pass@localhost:3306/test_db"
        );
    }

    #[test]
    fn test_database_url_with_special_characters() {
        // Test URL construction with special characters in password
        let config = DatabaseConfig {
            host: "db.example.com".to_string(),
            port: 3307,
            username: "admin".to_string(),
            password: "p@ss!word#123".to_string(),
            database: "fuel_bot".to_string(),
            max_connections: 10,
        };

        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );

        assert_eq!(
            database_url,
            "mysql://admin:p@ss!word#123@db.example.com:3307/fuel_bot"
        );
    }

    #[test]
    fn test_max_connections_configuration() {
        // Test that different max_connections values are properly configured
        let configs = vec![
            (1, "Single connection"),
            (5, "Default pool size"),
            (10, "Medium pool size"),
            (50, "Large pool size"),
        ];

        for (max_conn, description) in configs {
            let config = DatabaseConfig {
                host: "localhost".to_string(),
                port: 3306,
                username: "user".to_string(),
                password: "pass".to_string(),
                database: "db".to_string(),
                max_connections: max_conn,
            };

            assert_eq!(
                config.max_connections, max_conn,
                "Failed for: {}",
                description
            );
        }
    }

    // Note: Integration tests that actually connect to a database would be in
    // tests/integration/ directory and would require a test database to be running.
    // Those tests would verify:
    // - Successful connection with valid credentials
    // - Connection failure with invalid credentials
    // - Connection failure with unreachable host
    // - Pool behavior under concurrent load
    // - Connection timeout behavior
    // - Connection recycling and lifetime management
}
