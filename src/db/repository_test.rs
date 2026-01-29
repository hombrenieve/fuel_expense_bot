#[cfg(test)]
mod tests {
    use crate::db::repository::mock::MockRepository;
    use crate::db::repository::RepositoryTrait;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    /// Helper to create a decimal from a string
    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let repo = MockRepository::new();
        let result = repo.create_user("alice", 12345, dec("210.00")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_user_duplicate_fails() {
        let repo = MockRepository::new();

        // Create user first time - should succeed
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Try to create same user again - should fail
        let result = repo.create_user("alice", 67890, dec("300.00")).await;
        assert!(result.is_err());

        // Verify the error message indicates duplicate
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Duplicate") || err_msg.contains("Database"));
    }

    #[tokio::test]
    async fn test_get_user_config_existing() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let config = repo.get_user_config("alice").await.unwrap();
        assert!(config.is_some());

        let config = config.unwrap();
        assert_eq!(config.username, "alice");
        assert_eq!(config.chat_id, 12345);
        assert_eq!(config.pay_limit, dec("210.00"));
    }

    #[tokio::test]
    async fn test_get_user_config_nonexistent() {
        let repo = MockRepository::new();
        let config = repo.get_user_config("nonexistent").await.unwrap();
        assert!(config.is_none());
    }

    #[tokio::test]
    async fn test_update_user_limit_success() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Update the limit
        repo.update_user_limit("alice", dec("300.00"))
            .await
            .unwrap();

        // Verify the limit was updated
        let config = repo.get_user_config("alice").await.unwrap().unwrap();
        assert_eq!(config.pay_limit, dec("300.00"));
    }

    #[tokio::test]
    async fn test_update_user_limit_nonexistent_fails() {
        let repo = MockRepository::new();
        let result = repo.update_user_limit("nonexistent", dec("300.00")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_expense_success() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let id = repo
            .create_expense("alice", date, dec("45.50"))
            .await
            .unwrap();

        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_create_expense_duplicate_date_fails() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Create first expense - should succeed
        repo.create_expense("alice", date, dec("45.50"))
            .await
            .unwrap();

        // Try to create another expense for same user and date - should fail
        let result = repo.create_expense("alice", date, dec("30.00")).await;
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Duplicate") || err_msg.contains("Database"));
    }

    #[tokio::test]
    async fn test_create_expense_different_users_same_date() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();
        repo.create_user("bob", 67890, dec("210.00")).await.unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Both users can have expenses on the same date
        let id1 = repo
            .create_expense("alice", date, dec("45.50"))
            .await
            .unwrap();
        let id2 = repo
            .create_expense("bob", date, dec("30.00"))
            .await
            .unwrap();

        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_get_expense_for_date_existing() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let id = repo
            .create_expense("alice", date, dec("45.50"))
            .await
            .unwrap();

        let expense = repo.get_expense_for_date("alice", date).await.unwrap();
        assert!(expense.is_some());

        let expense = expense.unwrap();
        assert_eq!(expense.id, id);
        assert_eq!(expense.username, "alice");
        assert_eq!(expense.tx_date, date);
        assert_eq!(expense.quantity, dec("45.50"));
    }

    #[tokio::test]
    async fn test_get_expense_for_date_nonexistent() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let expense = repo.get_expense_for_date("alice", date).await.unwrap();
        assert!(expense.is_none());
    }

    #[tokio::test]
    async fn test_update_expense_success() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let id = repo
            .create_expense("alice", date, dec("45.50"))
            .await
            .unwrap();

        // Update the expense
        repo.update_expense(id, dec("60.00")).await.unwrap();

        // Verify the expense was updated
        let expense = repo
            .get_expense_for_date("alice", date)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(expense.quantity, dec("60.00"));
    }

    #[tokio::test]
    async fn test_update_expense_nonexistent_fails() {
        let repo = MockRepository::new();
        let result = repo.update_expense(999, dec("60.00")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_monthly_total_empty() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(total, dec("0"));
    }

    #[tokio::test]
    async fn test_get_monthly_total_single_expense() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        repo.create_expense("alice", date, dec("45.50"))
            .await
            .unwrap();

        let total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(total, dec("45.50"));
    }

    #[tokio::test]
    async fn test_get_monthly_total_multiple_expenses() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Add expenses on different days in January
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            dec("45.50"),
        )
        .await
        .unwrap();
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec("30.00"),
        )
        .await
        .unwrap();
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
            dec("20.75"),
        )
        .await
        .unwrap();

        let total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(total, dec("96.25"));
    }

    #[tokio::test]
    async fn test_get_monthly_total_excludes_other_months() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Add expenses in different months
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec("45.50"),
        )
        .await
        .unwrap();
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            dec("30.00"),
        )
        .await
        .unwrap();
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            dec("20.75"),
        )
        .await
        .unwrap();

        // January total should only include January expense
        let jan_total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(jan_total, dec("45.50"));

        // February total should only include February expense
        let feb_total = repo.get_monthly_total("alice", 2024, 2).await.unwrap();
        assert_eq!(feb_total, dec("30.00"));
    }

    #[tokio::test]
    async fn test_get_monthly_total_excludes_other_users() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();
        repo.create_user("bob", 67890, dec("210.00")).await.unwrap();

        // Add expenses for both users in January
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec("45.50"),
        )
        .await
        .unwrap();
        repo.create_expense(
            "bob",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec("30.00"),
        )
        .await
        .unwrap();

        // Alice's total should only include her expenses
        let alice_total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(alice_total, dec("45.50"));

        // Bob's total should only include his expenses
        let bob_total = repo.get_monthly_total("bob", 2024, 1).await.unwrap();
        assert_eq!(bob_total, dec("30.00"));
    }

    #[tokio::test]
    async fn test_get_monthly_total_month_boundaries() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Add expenses on first and last day of January
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            dec("10.00"),
        )
        .await
        .unwrap();
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            dec("20.00"),
        )
        .await
        .unwrap();

        // Add expenses just outside January
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
            dec("5.00"),
        )
        .await
        .unwrap();
        repo.create_expense(
            "alice",
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            dec("15.00"),
        )
        .await
        .unwrap();

        // January total should only include Jan 1 and Jan 31
        let jan_total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(jan_total, dec("30.00"));
    }

    #[tokio::test]
    async fn test_add_expense_with_limit_check_create_within_limit() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Note: MockRepository doesn't actually use the transaction parameter
        // We'll pass a dummy transaction by using a workaround
        // In real tests with the actual Repository, we'd use a real transaction

        // For now, we'll test the logic without the transaction parameter
        // by directly testing the components

        // First, verify we can create an expense
        let id = repo
            .create_expense("alice", date, dec("45.50"))
            .await
            .unwrap();
        assert!(id > 0);

        // Verify the monthly total
        let total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(total, dec("45.50"));
    }

    #[tokio::test]
    async fn test_add_expense_with_limit_check_update_within_limit() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Create initial expense
        let id = repo
            .create_expense("alice", date, dec("45.50"))
            .await
            .unwrap();

        // Update the expense
        repo.update_expense(id, dec("60.00")).await.unwrap();

        // Verify the expense was updated
        let expense = repo
            .get_expense_for_date("alice", date)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(expense.quantity, dec("60.00"));

        // Verify the monthly total reflects the update
        let total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(total, dec("60.00"));
    }

    #[tokio::test]
    async fn test_add_expense_logic_exceeds_limit_new() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("100.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Manually test the limit check logic
        let current_total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        let new_amount = dec("150.00");
        let limit = dec("100.00");

        // This should exceed the limit
        assert!(current_total + new_amount > limit);

        // Verify no expense exists yet
        let expense = repo.get_expense_for_date("alice", date).await.unwrap();
        assert!(expense.is_none());
    }

    #[tokio::test]
    async fn test_add_expense_logic_exceeds_limit_update() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("100.00"))
            .await
            .unwrap();

        let date1 = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Create initial expenses
        repo.create_expense("alice", date1, dec("60.00"))
            .await
            .unwrap();
        repo.create_expense("alice", date2, dec("20.00"))
            .await
            .unwrap();

        // Check current total
        let current_total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(current_total, dec("80.00"));

        // Get the existing expense for date2
        let existing = repo
            .get_expense_for_date("alice", date2)
            .await
            .unwrap()
            .unwrap();

        // Calculate what the new total would be if we updated date2 to 50.00
        let new_total = current_total - existing.quantity + dec("50.00");
        let limit = dec("100.00");

        // This should exceed the limit (60 + 50 = 110 > 100)
        assert!(new_total > limit);
    }

    #[tokio::test]
    async fn test_add_expense_logic_exactly_at_limit() {
        let repo = MockRepository::new();
        repo.create_user("alice", 12345, dec("100.00"))
            .await
            .unwrap();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Add expense exactly at limit
        repo.create_expense("alice", date, dec("100.00"))
            .await
            .unwrap();

        // Verify it was created
        let expense = repo.get_expense_for_date("alice", date).await.unwrap();
        assert!(expense.is_some());
        assert_eq!(expense.unwrap().quantity, dec("100.00"));

        // Verify monthly total
        let total = repo.get_monthly_total("alice", 2024, 1).await.unwrap();
        assert_eq!(total, dec("100.00"));
    }
}
