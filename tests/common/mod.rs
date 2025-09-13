use std::env;

/// Setup function for integration tests that require environment variables
pub fn setup_integration_test() {
    // Load .env file if it exists
    dotenvy::dotenv().ok();
}

/// Get test GitHub username from environment or use default
pub fn get_test_username() -> String {
    env::var("TEST_GITHUB_USERNAME").unwrap_or_else(|_| "samgozman".to_string())
}

/// Get a known invalid GitHub username for testing
pub fn get_invalid_username() -> String {
    "aaaaaaaaaaaaaaaaaaaaaabbbb".to_string()
}
