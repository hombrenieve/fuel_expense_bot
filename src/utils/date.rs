// Date handling utilities
// Implements requirements 6.1, 6.2, 6.3, 6.4

use chrono::{Datelike, Local, NaiveDate};

/// Get the current date using the system timezone
///
/// Requirement 6.3: Use system timezone for determining the current date
pub fn current_date() -> NaiveDate {
    Local::now().date_naive()
}

/// Get the first and last day of a given month
///
/// Returns a tuple of (first_day, last_day) for the specified year and month.
///
/// Requirement 6.1: Calculate monthly totals from first to last day of the month
///
/// # Arguments
/// * `year` - The year (e.g., 2024)
/// * `month` - The month (1-12)
///
/// # Panics
/// Panics if the month is not in the range 1-12
pub fn get_month_bounds(year: i32, month: u32) -> (NaiveDate, NaiveDate) {
    // First day of the month
    let first_day =
        NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid year/month combination");

    // Last day of the month - calculate by going to first day of next month and subtracting 1 day
    let last_day = if month == 12 {
        // December: last day is Dec 31
        NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
    } else {
        // For other months: get first day of next month and subtract 1 day
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    };

    (first_day, last_day)
}

/// Get the first and last day of the current month
///
/// Requirement 6.2: Automatically start tracking expenses for the new month
/// Requirement 6.3: Use system timezone for determining the current date
pub fn current_month_bounds() -> (NaiveDate, NaiveDate) {
    let today = current_date();
    get_month_bounds(today.year(), today.month())
}

/// Format a date for database storage
///
/// Returns the date in YYYY-MM-DD format, which is compatible with MySQL/MariaDB DATE type.
///
/// Requirement 6.4: Use date format compatible with database schema
pub fn format_date_for_db(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_current_date_returns_valid_date() {
        // Test that current_date returns a valid date
        let date = current_date();
        // Should be a reasonable year (between 2020 and 2100)
        assert!(date.year() >= 2020 && date.year() <= 2100);
        assert!(date.month() >= 1 && date.month() <= 12);
        assert!(date.day() >= 1 && date.day() <= 31);
    }

    #[test]
    fn test_get_month_bounds_january() {
        let (first, last) = get_month_bounds(2024, 1);
        assert_eq!(first, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(last, NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }

    #[test]
    fn test_get_month_bounds_february_leap_year() {
        let (first, last) = get_month_bounds(2024, 2);
        assert_eq!(first, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(last, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    #[test]
    fn test_get_month_bounds_february_non_leap_year() {
        let (first, last) = get_month_bounds(2023, 2);
        assert_eq!(first, NaiveDate::from_ymd_opt(2023, 2, 1).unwrap());
        assert_eq!(last, NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());
    }

    #[test]
    fn test_get_month_bounds_april_30_days() {
        let (first, last) = get_month_bounds(2024, 4);
        assert_eq!(first, NaiveDate::from_ymd_opt(2024, 4, 1).unwrap());
        assert_eq!(last, NaiveDate::from_ymd_opt(2024, 4, 30).unwrap());
    }

    #[test]
    fn test_get_month_bounds_december() {
        let (first, last) = get_month_bounds(2024, 12);
        assert_eq!(first, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        assert_eq!(last, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }

    #[test]
    fn test_current_month_bounds_returns_valid_bounds() {
        let (first, last) = current_month_bounds();
        let today = current_date();

        // First day should be the 1st of the current month
        assert_eq!(first.day(), 1);
        assert_eq!(first.month(), today.month());
        assert_eq!(first.year(), today.year());

        // Last day should be in the same month and year
        assert_eq!(last.month(), today.month());
        assert_eq!(last.year(), today.year());

        // Last day should be after first day
        assert!(last > first);

        // Today should be within bounds
        assert!(today >= first);
        assert!(today <= last);
    }

    #[test]
    fn test_format_date_for_db() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        assert_eq!(format_date_for_db(date), "2024-03-15");
    }

    #[test]
    fn test_format_date_for_db_single_digit_month_and_day() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        assert_eq!(format_date_for_db(date), "2024-01-05");
    }

    #[test]
    fn test_format_date_for_db_december_31() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert_eq!(format_date_for_db(date), "2024-12-31");
    }

    #[test]
    fn test_month_bounds_all_months_2024() {
        // Test all months in 2024 (leap year)
        let expected_days = vec![31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

        for month in 1..=12 {
            let (first, last) = get_month_bounds(2024, month);
            assert_eq!(first.day(), 1);
            assert_eq!(first.month(), month);
            assert_eq!(last.month(), month);
            assert_eq!(last.day(), expected_days[(month - 1) as usize]);
        }
    }

    #[test]
    fn test_date_format_round_trip() {
        // Test that we can parse back what we format
        let original = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let formatted = format_date_for_db(original);
        let parsed = NaiveDate::parse_from_str(&formatted, "%Y-%m-%d").unwrap();
        assert_eq!(original, parsed);
    }
}
