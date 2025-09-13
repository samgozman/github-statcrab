//! GitHub API Integration Tests
//!
//! These tests require a real GitHub API token to be set in the GITHUB_TOKEN environment variable.
//! They make real API calls to GitHub and test the actual functionality.
//!
//! Run with: cargo test --test github_api_integration

use github_statcrab::github::{GitHubApi, GitHubApiError};

mod common;

#[tokio::test]
async fn test_fetch_user_stats_real_user() {
    common::setup_integration_test();

    let api = GitHubApi::new();
    let username = common::get_test_username();

    let result = api.fetch_user_stats(&username).await;

    match result {
        Ok(stats) => {
            // Verify basic structure
            assert_eq!(stats.login, username);
            assert!(stats.name.is_some() || stats.name.is_none()); // Either way is valid

            // Merged PRs should not exceed total PRs
            assert!(stats.total_merged_prs <= stats.total_prs);

            println!(
                "✓ Successfully fetched stats for {}: {} stars, {} commits, {} PRs",
                stats.login, stats.total_stars, stats.total_commits_ytd, stats.total_prs
            );
        }
        Err(e) => panic!("Failed to fetch user stats: {:?}", e),
    }
}

#[tokio::test]
async fn test_fetch_user_stats_nonexistent_user() {
    common::setup_integration_test();

    let api = GitHubApi::new();
    let username = common::get_invalid_username();

    let result = api.fetch_user_stats(&username).await;

    match result {
        Err(GitHubApiError::UserNotFound) => {
            println!("✓ Correctly identified nonexistent user: {}", username);
        }
        Ok(_) => panic!(
            "Expected UserNotFound error but got success for user: {}",
            username
        ),
        Err(e) => panic!("Expected UserNotFound error but got: {:?}", e),
    }
}

#[tokio::test]
async fn test_invalid_username_validation() {
    common::setup_integration_test();

    let api = GitHubApi::new();

    // Test empty username
    match api.fetch_user_stats("").await {
        Err(GitHubApiError::InvalidUsername(msg)) => {
            assert!(msg.contains("empty"));
            println!("✓ Correctly rejected empty username");
        }
        other => panic!(
            "Expected InvalidUsername error for empty string, got: {:?}",
            other
        ),
    }

    // Test username with spaces
    match api.fetch_user_stats("user name").await {
        Err(GitHubApiError::InvalidUsername(msg)) => {
            assert!(msg.contains("spaces"));
            println!("✓ Correctly rejected username with spaces");
        }
        other => panic!(
            "Expected InvalidUsername error for username with spaces, got: {:?}",
            other
        ),
    }

    // Test username with invalid characters
    match api.fetch_user_stats("user@name").await {
        Err(GitHubApiError::InvalidUsername(msg)) => {
            assert!(msg.contains("invalid characters"));
            println!("✓ Correctly rejected username with invalid characters");
        }
        other => panic!(
            "Expected InvalidUsername error for invalid characters, got: {:?}",
            other
        ),
    }
}
