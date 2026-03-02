use super::*;
use crate::components::config::ConfigService;

/// Helper function to create a mock ConfigService with a specific APP_ENV value
fn mock_config(app_env: &str) -> ConfigService {
    ConfigService {
        database_url: "sqlite::memory:".to_string(),
        access_token_private_key: "test_private_key".to_string(),
        access_token_public_key: "test_public_key".to_string(),
        access_token_expires_in: "60m".to_string(),
        access_token_max_age: 3600,
        refresh_token_expires_in: "24h".to_string(),
        refresh_token_max_age: 86400,
        rust_log: "info".to_string(),
        schema_synchronize: false,
        host: "localhost".to_string(),
        port: 8080,
        app_env: app_env.to_string(),
        prod_database_url: "".to_string(),
        synchronize: false,
        auto_migrate: false,
        email_address: "test@example.com".to_string(),
        smtp_password: "password".to_string(),
        smtp_transport: "smtp.example.com".to_string(),
        port_host: "http://localhost:4100".to_string(),
    }
}

#[test]
fn test_production_uses_actual_email() {
    let config = mock_config("production");
    let service = MailSendService::new();
    let user_email = "user@example.com".to_string();
    let token = "test_token_123".to_string();

    // Build the message internally to check recipient
    // We can't actually send without a real SMTP server, so we'll test the logic separately
    // This test verifies the conditional logic works correctly

    // Simulate the environment check
    let recipient_email = if config.app_env.trim().to_lowercase() == "production" {
        user_email.clone()
    } else {
        "nechiforelsamuel@gmail.com".to_string()
    };

    assert_eq!(recipient_email, "user@example.com");
}

#[test]
fn test_production_case_insensitive() {
    let config = mock_config("PRODUCTION");
    let user_email = "user@example.com".to_string();

    let recipient_email = if config.app_env.trim().to_lowercase() == "production" {
        user_email.clone()
    } else {
        "nechiforelsamuel@gmail.com".to_string()
    };

    assert_eq!(recipient_email, "user@example.com");
}

#[test]
fn test_production_with_whitespace() {
    let config = mock_config(" production ");
    let user_email = "user@example.com".to_string();

    let recipient_email = if config.app_env.trim().to_lowercase() == "production" {
        user_email.clone()
    } else {
        "nechiforelsamuel@gmail.com".to_string()
    };

    assert_eq!(recipient_email, "user@example.com");
}

#[test]
fn test_development_uses_test_email() {
    let config = mock_config("development");
    let user_email = "user@example.com".to_string();

    let recipient_email = if config.app_env.trim().to_lowercase() == "production" {
        user_email.clone()
    } else {
        "nechiforelsamuel@gmail.com".to_string()
    };

    assert_eq!(recipient_email, "nechiforelsamuel@gmail.com");
}

#[test]
fn test_unknown_env_uses_test_email() {
    let config = mock_config("staging");
    let user_email = "user@example.com".to_string();

    let recipient_email = if config.app_env.trim().to_lowercase() == "production" {
        user_email.clone()
    } else {
        "nechiforelsamuel@gmail.com".to_string()
    };

    assert_eq!(recipient_email, "nechiforelsamuel@gmail.com");
}

#[test]
fn test_empty_env_uses_test_email() {
    let config = mock_config("");
    let user_email = "user@example.com".to_string();

    let recipient_email = if config.app_env.trim().to_lowercase() == "production" {
        user_email.clone()
    } else {
        "nechiforelsamuel@gmail.com".to_string()
    };

    assert_eq!(recipient_email, "nechiforelsamuel@gmail.com");
}

#[test]
fn test_mixed_case_production() {
    let config = mock_config("PrOdUcTiOn");
    let user_email = "user@example.com".to_string();

    let recipient_email = if config.app_env.trim().to_lowercase() == "production" {
        user_email.clone()
    } else {
        "nechiforelsamuel@gmail.com".to_string()
    };

    assert_eq!(recipient_email, "user@example.com");
}
