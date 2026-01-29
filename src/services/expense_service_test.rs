#[cfg(test)]
mod tests {
    use crate::db::repository::mock::MockRepository;
    use crate::db::repository::RepositoryTrait;
    use crate::services::expense_service::{AddExpenseResult, ExpenseService};
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use std::sync::Arc;

    /// Helper to create a decimal from a string
    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    /// Helper to create an ExpenseService with a mock repository
    fn create_service() -> (ExpenseService, Arc<MockRepository>) {
        let repo = Arc::new(MockRepository::new());
        let service = ExpenseService::new(repo.clone());
        (service, repo)
    }

    #[tokio::test]
    async fn test_add_expense_user_not_found() {
        let (service, _repo) = create_service();

        // Try to add expense for non-existent user
        let result = service.add_expense("nonexistent", dec("45.50")).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::utils::error::BotError::UserNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_add_expense_success_new() {
        let (service, repo) = create_service();

        // Create user
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Add expense
        let result = service.add_expense("alice", dec("45.50")).await.unwrap();

        // Should succeed
        match result {
            AddExpenseResult::Success {
                new_total,
                remaining,
            } => {
                assert_eq!(new_total, dec("45.50"));
                assert_eq!(remaining, dec("164.50")); // 210 - 45.50
            }
            _ => panic!("Expected Success result"),
        }
    }

    #[tokio::test]
    async fn test_add_expense_success_update_same_day() {
        let (service, repo) = create_service();

        // Create user
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Add first expense
        let result1 = service.add_expense("alice", dec("45.50")).await.unwrap();
        match result1 {
            AddExpenseResult::Success {
                new_total,
                remaining,
            } => {
                assert_eq!(new_total, dec("45.50"));
                assert_eq!(remaining, dec("164.50"));
            }
            _ => panic!("Expected Success result"),
        }

        // Add second expense on same day (should add to existing)
        let result2 = service.add_expense("alice", dec("30.00")).await.unwrap();
        match result2 {
            AddExpenseResult::Success {
                new_total,
                remaining,
            } => {
                assert_eq!(new_total, dec("75.50")); // 45.50 + 30.00
                assert_eq!(remaining, dec("134.50")); // 210 - 75.50
            }
            _ => panic!("Expected Success result"),
        }
    }

    #[tokio::test]
    async fn test_add_expense_limit_exceeded_new() {
        let (service, repo) = create_service();

        // Create user with low limit
        repo.create_user("alice", 12345, dec("100.00"))
            .await
            .unwrap();

        // Try to add expense that exceeds limit
        let result = service.add_expense("alice", dec("150.00")).await.unwrap();

        // Should be rejected
        match result {
            AddExpenseResult::LimitExceeded {
                current,
                attempted,
                limit,
            } => {
                assert_eq!(current, dec("0"));
                assert_eq!(attempted, dec("150.00"));
                assert_eq!(limit, dec("100.00"));
            }
            _ => panic!("Expected LimitExceeded result"),
        }
    }

    #[tokio::test]
    async fn test_add_expense_limit_exceeded_update() {
        let (service, repo) = create_service();

        // Create user
        repo.create_user("alice", 12345, dec("100.00"))
            .await
            .unwrap();

        // Add first expense
        service.add_expense("alice", dec("60.00")).await.unwrap();

        // Try to add second expense that would exceed limit
        let result = service.add_expense("alice", dec("50.00")).await.unwrap();

        // Should be rejected
        match result {
            AddExpenseResult::LimitExceeded {
                current,
                attempted,
                limit,
            } => {
                assert_eq!(current, dec("60.00"));
                assert_eq!(attempted, dec("50.00"));
                assert_eq!(limit, dec("100.00"));
            }
            _ => panic!("Expected LimitExceeded result"),
        }
    }

    #[tokio::test]
    async fn test_add_expense_exactly_at_limit() {
        let (service, repo) = create_service();

        // Create user
        repo.create_user("alice", 12345, dec("100.00"))
            .await
            .unwrap();

        // Add expense exactly at limit
        let result = service.add_expense("alice", dec("100.00")).await.unwrap();

        // Should succeed
        match result {
            AddExpenseResult::Success {
                new_total,
                remaining,
            } => {
                assert_eq!(new_total, dec("100.00"));
                assert_eq!(remaining, dec("0"));
            }
            _ => panic!("Expected Success result"),
        }
    }

    #[tokio::test]
    async fn test_get_monthly_summary_no_expenses() {
        let (service, repo) = create_service();

        // Create user
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Get summary
        let summary = service.get_monthly_summary("alice").await.unwrap();

        assert_eq!(summary.total_spent, dec("0"));
        assert_eq!(summary.limit, dec("210.00"));
        assert_eq!(summary.remaining, dec("210.00"));
    }

    #[tokio::test]
    async fn test_get_monthly_summary_with_expenses() {
        let (service, repo) = create_service();

        // Create user
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Add some expenses (we need to add them directly to repo since service uses current date)
        let today = crate::utils::date::current_date();
        repo.create_expense("alice", today, dec("45.50"))
            .await
            .unwrap();

        // Get summary
        let summary = service.get_monthly_summary("alice").await.unwrap();

        assert_eq!(summary.total_spent, dec("45.50"));
        assert_eq!(summary.limit, dec("210.00"));
        assert_eq!(summary.remaining, dec("164.50"));
    }

    #[tokio::test]
    async fn test_get_monthly_summary_user_not_found() {
        let (service, _repo) = create_service();

        // Try to get summary for non-existent user
        let result = service.get_monthly_summary("nonexistent").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::utils::error::BotError::UserNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_summary_arithmetic_correctness() {
        let (service, repo) = create_service();

        // Create user
        repo.create_user("alice", 12345, dec("210.00"))
            .await
            .unwrap();

        // Add expenses
        let today = crate::utils::date::current_date();
        repo.create_expense("alice", today, dec("75.25"))
            .await
            .unwrap();

        // Get summary
        let summary = service.get_monthly_summary("alice").await.unwrap();

        // Verify arithmetic: remaining = limit - total_spent
        assert_eq!(summary.remaining, summary.limit - summary.total_spent);
        assert_eq!(summary.remaining, dec("134.75"));
    }
}

#[cfg(test)]
mod property_tests {
    use crate::db::repository::mock::MockRepository;
    use crate::db::repository::RepositoryTrait;
    use crate::services::expense_service::{AddExpenseResult, ExpenseService};
    use chrono::{Datelike, NaiveDate};
    use proptest::prelude::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use std::sync::Arc;

    /// Helper to create a decimal from a string
    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    /// Helper to create an ExpenseService with a mock repository
    fn create_service() -> (ExpenseService, Arc<MockRepository>) {
        let repo = Arc::new(MockRepository::new());
        let service = ExpenseService::new(repo.clone());
        (service, repo)
    }

    /// Strategy for generating valid usernames (alphanumeric, 1-50 chars)
    fn username_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9]{1,50}"
    }

    /// Strategy for generating valid expense amounts (0.01 to 9999.99)
    fn amount_strategy() -> impl Strategy<Value = Decimal> {
        (1u64..=999999u64).prop_map(|cents| Decimal::from(cents) / dec("100"))
    }

    /// Strategy for generating valid limits (50.00 to 500.00)
    fn limit_strategy() -> impl Strategy<Value = Decimal> {
        (5000u64..=50000u64).prop_map(|cents| Decimal::from(cents) / dec("100"))
    }

    /// Strategy for generating dates in 2024
    fn date_strategy() -> impl Strategy<Value = NaiveDate> {
        (1u32..=12u32, 1u32..=28u32)
            .prop_map(|(month, day)| NaiveDate::from_ymd_opt(2024, month, day).unwrap())
    }

    proptest! {
        /// Feature: rust-telegram-bot-migration, Property 4: Expense Date Assignment
        ///
        /// For any expense added without an explicit date, the system should assign
        /// the current system date as the transaction date.
        ///
        /// **Validates: Requirements 2.2**
        #[test]
        fn property_4_expense_date_assignment(
            username in username_strategy(),
            amount in amount_strategy(),
            limit in limit_strategy(),
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (service, repo) = create_service();

                // Create user
                repo.create_user(&username, 12345, limit).await.unwrap();

                // Get current date before adding expense
                let expected_date = crate::utils::date::current_date();

                // Add expense (uses current date internally)
                let result = service.add_expense(&username, amount).await.unwrap();

                // Verify expense was added
                if let AddExpenseResult::Success { .. } = result {
                    // Check that an expense exists for today's date
                    let expense = repo.get_expense_for_date(&username, expected_date).await.unwrap();
                    prop_assert!(expense.is_some(), "Expense should exist for current date");

                    let expense = expense.unwrap();
                    prop_assert_eq!(expense.tx_date, expected_date, "Expense date should be current date");
                }

                Ok(())
            })?;
        }

        /// Feature: rust-telegram-bot-migration, Property 5: Limit Enforcement
        ///
        /// For any user with a monthly limit L and current month total T, when attempting
        /// to add an expense E:
        /// - If T + E > L, the system should reject the expense
        /// - If T + E ≤ L, the system should accept the expense
        ///
        /// **Validates: Requirements 2.3, 2.6**
        #[test]
        fn property_5_limit_enforcement(
            username in username_strategy(),
            limit in limit_strategy(),
            expense1 in amount_strategy(),
            expense2 in amount_strategy(),
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (service, repo) = create_service();

                // Create user with specified limit
                repo.create_user(&username, 12345, limit).await.unwrap();

                // Add first expense if it's within limit
                if expense1 <= limit {
                    let result1 = service.add_expense(&username, expense1).await.unwrap();
                    prop_assert!(
                        matches!(result1, AddExpenseResult::Success { .. }),
                        "First expense within limit should succeed"
                    );

                    // Try to add second expense (will be added to first on same day)
                    let result2 = service.add_expense(&username, expense2).await.unwrap();

                    // Check if second expense should be accepted or rejected
                    // Second expense is added to first (same day), so check combined total
                    if expense1 + expense2 <= limit {
                        // Combined expenses within limit
                        prop_assert!(
                            matches!(result2, AddExpenseResult::Success { .. }),
                            "Second expense within limit should succeed"
                        );
                    } else {
                        // Combined expenses exceed limit
                        prop_assert!(
                            matches!(result2, AddExpenseResult::LimitExceeded { .. }),
                            "Second expense exceeding limit should be rejected"
                        );
                    }
                } else {
                    // First expense exceeds limit
                    let result1 = service.add_expense(&username, expense1).await.unwrap();
                    prop_assert!(
                        matches!(result1, AddExpenseResult::LimitExceeded { .. }),
                        "First expense exceeding limit should be rejected"
                    );
                }

                Ok(())
            })?;
        }

        /// Feature: rust-telegram-bot-migration, Property 6: Successful Expense Addition
        ///
        /// For any user and valid expense amount within the monthly limit, adding the
        /// expense should result in:
        /// - If no expense exists for that date: a new expense record is created
        /// - If an expense exists for that date: the existing expense is updated
        /// - The monthly total increases by the added amount
        /// - The operation completes atomically
        ///
        /// **Validates: Requirements 2.4, 2.5, 5.1**
        #[test]
        fn property_6_successful_expense_addition(
            username in username_strategy(),
            limit in limit_strategy(),
            amount in amount_strategy(),
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (service, repo) = create_service();

                // Create user
                repo.create_user(&username, 12345, limit).await.unwrap();

                // Only test with amounts within limit
                if amount <= limit {
                    // Get initial monthly total
                    let today = crate::utils::date::current_date();
                    let year = today.year();
                    let month = today.month();
                    let initial_total = repo.get_monthly_total(&username, year, month).await.unwrap();

                    // Add expense
                    let result = service.add_expense(&username, amount).await.unwrap();

                    // Should succeed
                    prop_assert!(
                        matches!(result, AddExpenseResult::Success { .. }),
                        "Expense within limit should succeed"
                    );

                    // Verify expense exists for today
                    let expense = repo.get_expense_for_date(&username, today).await.unwrap();
                    prop_assert!(expense.is_some());

                    let expense = expense.unwrap();
                    prop_assert_eq!(expense.quantity, amount);

                    // Verify monthly total increased correctly
                    let new_total = repo.get_monthly_total(&username, year, month).await.unwrap();
                    prop_assert_eq!(new_total, initial_total + amount);
                }

                Ok(())
            })?;
        }

        /// Feature: rust-telegram-bot-migration, Property 7: Monthly Total Calculation
        ///
        /// For any set of expenses for a user, the monthly total should equal the sum
        /// of all expense quantities where the transaction date falls within the current
        /// calendar month boundaries.
        ///
        /// **Validates: Requirements 3.1, 6.1**
        #[test]
        fn property_7_monthly_total_calculation(
            username in username_strategy(),
            _limit in limit_strategy(),
            expenses in prop::collection::vec(amount_strategy(), 1..5),
            dates in prop::collection::vec(date_strategy(), 1..5),
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (_service, repo) = create_service();

                // Create user with high limit to avoid rejections
                let high_limit = dec("10000.00");
                repo.create_user(&username, 12345, high_limit).await.unwrap();

                // Add expenses on different dates
                let mut expected_totals: std::collections::HashMap<(i32, u32), Decimal> =
                    std::collections::HashMap::new();

                for (i, &amount) in expenses.iter().enumerate() {
                    let date = dates[i % dates.len()];
                    let year = date.year();
                    let month = date.month();

                    // Try to create expense (might fail if date already used)
                    if repo.create_expense(&username, date, amount).await.is_ok() {
                        *expected_totals.entry((year, month)).or_insert(dec("0")) += amount;
                    }
                }

                // Verify monthly totals for each month
                for ((year, month), expected_total) in expected_totals {
                    let actual_total = repo.get_monthly_total(&username, year, month).await.unwrap();
                    prop_assert_eq!(actual_total, expected_total,
                        "Monthly total for {}-{} should match sum of expenses", year, month);
                }

                Ok(())
            })?;
        }

        /// Feature: rust-telegram-bot-migration, Property 8: Summary Arithmetic Correctness
        ///
        /// For any user's monthly summary, the following must hold:
        /// - remaining = limit - total_spent (arithmetic invariant)
        ///
        /// **Validates: Requirements 3.2, 3.3, 3.4**
        #[test]
        fn property_8_summary_arithmetic_correctness(
            username in username_strategy(),
            limit in limit_strategy(),
            expenses in prop::collection::vec(amount_strategy(), 0..5),
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (service, repo) = create_service();

                // Create user
                repo.create_user(&username, 12345, limit).await.unwrap();

                // Add expenses for current month
                let today = crate::utils::date::current_date();
                let mut total_added = dec("0");

                for (i, &amount) in expenses.iter().enumerate() {
                    // Use different days to avoid conflicts
                    let day = (today.day() + i as u32) % 28 + 1;
                    if let Some(date) = NaiveDate::from_ymd_opt(today.year(), today.month(), day) {
                        if total_added + amount <= limit {
                            if repo.create_expense(&username, date, amount).await.is_ok() {
                                total_added += amount;
                            }
                        }
                    }
                }

                // Get summary
                let summary = service.get_monthly_summary(&username).await.unwrap();

                // Verify arithmetic invariant
                prop_assert_eq!(summary.remaining, summary.limit - summary.total_spent,
                    "Remaining should equal limit minus total spent");

                // Verify values
                prop_assert_eq!(summary.limit, limit);
                prop_assert_eq!(summary.total_spent, total_added);

                Ok(())
            })?;
        }
    }
}
