use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Europe;
use nanoid::nanoid;
use sea_orm::DbErr;

use crate::http_response::{error_handler::CustomError, HttpCodeW};

/// Generates a random ID for authentication purposes
#[allow(dead_code)]
pub fn generate_auth_id() -> String {
    nanoid!(32) // Generate a 32-character ID for secure authentication
}

/// Generates a secure token for authentication
#[allow(dead_code)]
pub fn generate_secure_token() -> String {
    nanoid!(64, &[
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
        'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
        'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'
    ])
}

/// Checks if a database error is due to a duplicate key constraint violation.
///
/// This function examines the database operation result and determines if it failed
/// due to a unique constraint violation. If so, it increments the attempts counter
/// to facilitate retry logic. For other errors, it converts them to CustomError.
///
/// # Type Parameters
///
/// * `T`: The type of the successful result (e.g., database model, entity)
///
/// # Arguments
///
/// * `attempts`: Mutable reference to the retry counter that gets incremented on duplicate key errors
/// * `result`: The database operation result to examine (Result<T, DbErr>)
///
/// returns: Option<Result<T, CustomError>> - Some(Ok(value)) on success, Some(Err(error)) on non-duplicate errors, None on duplicate key (for retry)
///
/// # Examples
///
/// ```rust
/// let mut retry_count = 0;
/// let db_result: Result<User, DbErr> = user_repository.create(new_user).await;
///
/// if let Some(final_result) = check_if_is_duplicate_key(&mut retry_count, db_result) {
///     return final_result; // Either success or non-retryable error
/// }
/// // If None returned, it was a duplicate key - continue with retry logic
///
/// // Works with any type
/// let product_result: Result<Product, DbErr> = product_repository.create(new_product).await;
/// if let Some(final_result) = check_if_is_duplicate_key(&mut retry_count, product_result) {
///     return final_result;
/// }
/// ```
#[allow(dead_code)]
pub fn check_if_is_duplicate_key_from_data_base<T>(
    attempts: &mut usize,
    result: Result<T, DbErr>,
) -> Option<Result<T, CustomError>> {
    match result {
        Ok(value) => Some(Ok(value)),
        Err(e) => {
            // Check if the error is a unique constraint violation
            // The exact string to check for might vary slightly depending on the database
            println!("Error occurred while checking for duplicate key: {e}");
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                // It's a unique constraint violation, increment attempts for retry logic
                *attempts += 1;
                // Return None to indicate this is a retryable duplicate key error
                None
            } else {
                // Some other execution error, return it
                Some(Err(CustomError::from(e)))
            }
        }
    }
}

#[allow(dead_code)]
pub fn now_time() -> NaiveDateTime {
    chrono::Utc::now()
        .with_timezone(&Europe::Bucharest)
        .naive_local()
}

/// Parses a date string into a DateTime<Utc> object.
///
/// This function attempts to parse the string using common date formats:
/// - ISO 8601: "2025-07-22T14:30:00"
/// - Standard format: "2025-07-22 14:30:00"
/// - Alternative formats if needed
///
/// # Arguments
///
/// * `date_str`: The date string to parse
///
/// # Returns
///
/// * `Result<DateTime<Utc>, CustomError>`: The parsed DateTime or a formatting error
///
/// # Examples
///
/// ```rust
/// let date = parse_date("2025-07-22 14:30:00")?;
/// ```
#[allow(dead_code)]
pub fn parse_date(date_str: &str) -> Result<DateTime<Utc>, CustomError> {
    // Try parsing as standard format "YYYY-MM-DD HH:MM:SS"
    if let Ok(naive) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
        return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
    }

    // Try parsing as ISO 8601 format
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try alternative format "DD/MM/YYYY HH:MM:SS"
    if let Ok(naive) = NaiveDateTime::parse_from_str(date_str, "%d/%m/%Y %H:%M:%S") {
        return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
    }

    // Try date-only format and assume midnight time
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        if let Some(datetime) = date.and_hms_opt(0, 0, 0) {
            return Ok(DateTime::from_naive_utc_and_offset(datetime, Utc));
        }
    }

    // If all parsing attempts fail, return an error
    Err(CustomError::new(
        HttpCodeW::BadRequest,
        format!("Unable to parse date: {date_str}"),
    ))
}

/// Validates an email address format
#[allow(dead_code)]
pub fn validate_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

/// Validates password strength
#[allow(dead_code)]
pub fn validate_password(password: &str) -> Result<(), CustomError> {
    if password.len() < 8 {
        return Err(CustomError::new(
            HttpCodeW::BadRequest,
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(CustomError::new(
            HttpCodeW::BadRequest,
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(CustomError::new(
            HttpCodeW::BadRequest,
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(CustomError::new(
            HttpCodeW::BadRequest,
            "Password must contain at least one digit".to_string(),
        ));
    }

    Ok(())
}

/// Generates a salt for password hashing
#[allow(dead_code)]
pub fn generate_salt() -> String {
    nanoid!(16)
}
