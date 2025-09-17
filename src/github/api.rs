use reqwest::Client;
use serde_json::json;
use std::env;
use std::sync::OnceLock;
use std::sync::{Arc, RwLock};

use crate::github::cache::get_github_cache;
use crate::github::types::*;

#[derive(Debug, Clone, Default)]
pub struct GitHubRateLimit {
    pub limit: Option<u64>,
    pub remaining: Option<u64>,
    pub used: Option<u64>,
    pub reset: Option<u64>,
}

// Global rate limit state
static RATE_LIMIT_STATE: OnceLock<Arc<RwLock<GitHubRateLimit>>> = OnceLock::new();

fn get_rate_limit_state() -> Arc<RwLock<GitHubRateLimit>> {
    RATE_LIMIT_STATE
        .get_or_init(|| Arc::new(RwLock::new(GitHubRateLimit::default())))
        .clone()
}

/// Get the current GitHub rate limit information
pub fn get_github_rate_limit() -> GitHubRateLimit {
    let state = get_rate_limit_state();
    let guard = state.read().unwrap_or_else(|poisoned| {
        // If the lock is poisoned, we still want to get the data
        poisoned.into_inner()
    });
    guard.clone()
}

/// Update the GitHub rate limit information from response headers
fn update_rate_limit_from_headers(headers: &reqwest::header::HeaderMap) {
    let state = get_rate_limit_state();
    let mut guard = state.write().unwrap_or_else(|poisoned| {
        // If the lock is poisoned, we still want to update the data
        poisoned.into_inner()
    });

    // Parse rate limit headers
    guard.limit = headers
        .get("x-ratelimit-limit")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    guard.remaining = headers
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    guard.used = headers
        .get("x-ratelimit-used")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    guard.reset = headers
        .get("x-ratelimit-reset")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());
}

/// Check if we should make a GitHub API request based on current rate limits
fn check_rate_limit_before_request() -> Result<(), GitHubApiError> {
    let rate_limit = get_github_rate_limit();
    check_rate_limit_with_data(&rate_limit)
}

/// Check if we should make a GitHub API request based on provided rate limit data
fn check_rate_limit_with_data(rate_limit: &GitHubRateLimit) -> Result<(), GitHubApiError> {
    // If we don't have rate limit info yet, allow the request
    if rate_limit.remaining.is_none() || rate_limit.reset.is_none() {
        return Ok(());
    }

    let remaining = rate_limit.remaining.unwrap();
    let reset_time = rate_limit.reset.unwrap();

    // Check if remaining requests are below threshold
    if remaining < 100 {
        // Check if we're still within the rate limit window
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if current_time < reset_time {
            return Err(GitHubApiError::RateLimitProtection(remaining, reset_time));
        }

        // If the reset time has passed, allow the request
        // (the rate limit will be updated after the request)
    }

    Ok(())
}

#[derive(Debug)]
pub struct GitHubApi {
    client: Client,
    token: Option<String>,
}

impl Default for GitHubApi {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubApi {
    /// Create a new GitHub API client
    pub fn new() -> Self {
        let client = Client::new();
        let token = env::var("GITHUB_TOKEN").ok();

        Self { client, token }
    }

    /// Validate username format
    fn validate_username(username: &str) -> Result<(), GitHubApiError> {
        if username.trim().is_empty() {
            return Err(GitHubApiError::InvalidUsername(
                "Username cannot be empty".to_string(),
            ));
        }
        if username.contains(' ') {
            return Err(GitHubApiError::InvalidUsername(
                "Username cannot contain spaces".to_string(),
            ));
        }
        if username.len() > 39 {
            return Err(GitHubApiError::InvalidUsername(
                "Username too long".to_string(),
            ));
        }

        // Basic GitHub username validation
        let valid_chars = username.chars().all(|c| c.is_alphanumeric() || c == '-');
        if !valid_chars {
            return Err(GitHubApiError::InvalidUsername(
                "Username contains invalid characters".to_string(),
            ));
        }

        if username.starts_with('-') || username.ends_with('-') {
            return Err(GitHubApiError::InvalidUsername(
                "Username cannot start or end with hyphen".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the GraphQL query for fetching user stats
    fn get_stats_query() -> String {
        r#"
        query GetUserStats($login: String!, $after: String) {
            user(login: $login) {
                name
                login
                contributionsCollection {
                    totalCommitContributions
                    totalPullRequestReviewContributions
                }
                pullRequests(first: 1) {
                    totalCount
                }
                mergedPullRequests: pullRequests(states: MERGED) {
                    totalCount
                }
                openIssues: issues(states: OPEN) {
                    totalCount
                }
                closedIssues: issues(states: CLOSED) {
                    totalCount
                }
                repositoryDiscussions {
                    totalCount
                }
                repositoryDiscussionComments(onlyAnswers: true) {
                    totalCount
                }
                repositories(first: 100, ownerAffiliations: OWNER, orderBy: {direction: DESC, field: STARGAZERS}, after: $after) {
                    totalCount
                    nodes {
                        name
                        stargazers {
                            totalCount
                        }
                    }
                    pageInfo {
                        hasNextPage
                        endCursor
                    }
                }
            }
        }
        "#.to_string()
    }

    /// Get the GraphQL query for fetching additional repositories (pagination)
    fn get_repos_query() -> String {
        r#"
        query GetUserRepos($login: String!, $after: String) {
            user(login: $login) {
                repositories(first: 100, ownerAffiliations: OWNER, orderBy: {direction: DESC, field: STARGAZERS}, after: $after) {
                    totalCount
                    nodes {
                        name
                        stargazers {
                            totalCount
                        }
                    }
                    pageInfo {
                        hasNextPage
                        endCursor
                    }
                }
            }
        }
        "#.to_string()
    }

    /// Get the GraphQL query for fetching user languages
    fn get_languages_query() -> String {
        r#"
        query GetUserLanguages($login: String!, $after: String) {
            user(login: $login) {
                repositories(ownerAffiliations: OWNER, isFork: false, first: 100, after: $after) {
                    nodes {
                        name
                        languages(first: 10, orderBy: {field: SIZE, direction: DESC}) {
                            edges {
                                size
                                node {
                                    color
                                    name
                                }
                            }
                        }
                    }
                    pageInfo {
                        hasNextPage
                        endCursor
                    }
                }
            }
        }
        "#
        .to_string()
    }

    /// Execute a GraphQL query
    #[tracing::instrument(name = "github_api_request", skip(self, query, variables))]
    async fn execute_query<T>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<GraphQLResponse<T>, GitHubApiError>
    where
        T: serde::de::DeserializeOwned,
    {
        let token = self.token.as_ref().ok_or(GitHubApiError::MissingToken)?;

        // Check rate limit before making the request
        check_rate_limit_before_request()?;

        let payload = json!({
            "query": query,
            "variables": variables
        });

        // Add Sentry context for the API request
        sentry::configure_scope(|scope| {
            scope.set_tag("github_api", "graphql");
            scope.set_context(
                "github_request",
                sentry::protocol::Context::Other({
                    let mut map = std::collections::BTreeMap::new();
                    map.insert(
                        "endpoint".to_string(),
                        "https://api.github.com/graphql".into(),
                    );
                    map.insert("variables".to_string(), variables.to_string().into());
                    map
                }),
            );
        });

        let response = self
            .client
            .post("https://api.github.com/graphql")
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", "github-statcrab")
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                // Report network errors to Sentry
                sentry::capture_error(&e);
                tracing::error!("GitHub API network error: {e}");
                GitHubApiError::NetworkError(e)
            })?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(GitHubApiError::MissingToken);
        }

        // Update rate limit information from response headers
        update_rate_limit_from_headers(response.headers());

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            // Rate limit info for debugging
            let reset_time = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown");

            sentry::configure_scope(|scope| {
                scope.set_extra("rate_limit_reset", reset_time.into());
            });

            return Err(GitHubApiError::RateLimitExceeded);
        }

        // Check for other HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_msg = format!("GitHub API returned HTTP {status}");
            sentry::capture_message(&error_msg, sentry::Level::Error);
            tracing::error!("{error_msg}");
        }

        let response_body: GraphQLResponse<T> = response.json().await.map_err(|e| {
            sentry::capture_error(&e);
            tracing::error!("Failed to parse GitHub API response: {e}");
            GitHubApiError::NetworkError(e)
        })?;

        Ok(response_body)
    }

    /// Fetch user statistics from GitHub
    #[tracing::instrument(name = "fetch_user_stats", fields(username = %username))]
    pub async fn fetch_user_stats(&self, username: &str) -> Result<GitHubStats, GitHubApiError> {
        Self::validate_username(username)?;

        let cache = get_github_cache();
        let username_owned = username.to_string();
        let api_ref = self;

        cache
            .get_or_insert_user_stats(username_owned.clone(), || async move {
                api_ref.fetch_user_stats_uncached(&username_owned).await
            })
            .await
    }

    /// Fetch user statistics from GitHub without caching
    #[tracing::instrument(name = "fetch_user_stats_uncached", fields(username = %username))]
    async fn fetch_user_stats_uncached(
        &self,
        username: &str,
    ) -> Result<GitHubStats, GitHubApiError> {
        Self::validate_username(username)?;

        // Initial query to get basic stats and first page of repositories
        let variables = json!({
            "login": username,
            "after": null,
        });

        let query = Self::get_stats_query();
        let response: GraphQLResponse<UserQueryResponse> =
            self.execute_query(&query, variables).await?;

        // Handle GraphQL errors
        if let Some(errors) = response.errors
            && let Some(error) = errors.first()
        {
            if error.error_type.as_deref() == Some("NOT_FOUND") {
                return Err(GitHubApiError::UserNotFound);
            }
            return Err(GitHubApiError::GraphQLError(error.message.clone()));
        }

        let user_response = response.data.ok_or(GitHubApiError::GraphQLError(
            "No data in response".to_string(),
        ))?;
        let user = user_response.user.ok_or(GitHubApiError::UserNotFound)?;

        // Collect all repositories (handle pagination)
        let mut all_repositories = user.repositories.nodes.clone();
        let mut has_next_page = user.repositories.page_info.has_next_page;
        let mut end_cursor = user.repositories.page_info.end_cursor.clone();

        // Fetch additional pages of repositories if needed
        while has_next_page {
            let variables = json!({
                "login": username,
                "after": end_cursor
            });

            let repos_query = Self::get_repos_query();
            let repos_response: GraphQLResponse<UserQueryResponse> =
                self.execute_query(&repos_query, variables).await?;

            if let Some(data) = repos_response.data {
                if let Some(user_data) = data.user {
                    all_repositories.extend(user_data.repositories.nodes);
                    has_next_page = user_data.repositories.page_info.has_next_page;
                    end_cursor = user_data.repositories.page_info.end_cursor;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Calculate total stars
        let total_stars = all_repositories
            .iter()
            .map(|repo| repo.stargazers.total_count)
            .sum();

        // Build the final stats
        let stats = GitHubStats {
            name: user.name,
            login: user.login,
            total_stars,
            total_commits_ytd: user.contributions_collection.total_commit_contributions,
            total_prs: user.pull_requests.total_count,
            total_merged_prs: user.merged_pull_requests.map_or(0, |mrs| mrs.total_count),
            total_reviews: user
                .contributions_collection
                .total_pull_request_review_contributions,
            total_issues: user.open_issues.total_count + user.closed_issues.total_count,
            total_discussions_started: user.repository_discussions.map_or(0, |rd| rd.total_count),
            total_discussions_answered: user
                .repository_discussion_comments
                .map_or(0, |rdc| rdc.total_count),
        };

        Ok(stats)
    }

    /// Fetch user languages from GitHub
    #[tracing::instrument(name = "fetch_user_languages", fields(username = %username, excluded_repos = exclude_repos.len()))]
    pub async fn fetch_user_languages(
        &self,
        username: &str,
        exclude_repos: &[String],
    ) -> Result<Vec<crate::cards::langs_card::LanguageStat>, GitHubApiError> {
        Self::validate_username(username)?;

        let cache = get_github_cache();
        let username_owned = username.to_string();
        let exclude_repos_owned = exclude_repos.to_vec();
        let api_ref = self;

        cache
            .get_or_insert_user_languages(username_owned.clone(), &exclude_repos_owned, || {
                let exclude_repos_cloned = exclude_repos_owned.clone();
                async move {
                    api_ref
                        .fetch_user_languages_uncached(&username_owned, &exclude_repos_cloned)
                        .await
                }
            })
            .await
    }

    /// Fetch user languages from GitHub without caching
    #[tracing::instrument(name = "fetch_user_languages_uncached", fields(username = %username, excluded_repos = exclude_repos.len()))]
    async fn fetch_user_languages_uncached(
        &self,
        username: &str,
        exclude_repos: &[String],
    ) -> Result<Vec<crate::cards::langs_card::LanguageStat>, GitHubApiError> {
        let mut all_repos = Vec::new();
        let mut after_cursor: Option<String> = None;
        let mut has_next_page = true;

        // Fetch all repositories with languages (handle pagination)
        while has_next_page {
            let variables = json!({
                "login": username,
                "after": after_cursor
            });

            let query = Self::get_languages_query();
            let response: GraphQLResponse<LanguagesQueryResponse> =
                self.execute_query(&query, variables).await?;

            // Handle GraphQL errors
            if let Some(errors) = response.errors
                && let Some(error) = errors.first()
            {
                if error.error_type.as_deref() == Some("NOT_FOUND") {
                    return Err(GitHubApiError::UserNotFound);
                }
                return Err(GitHubApiError::GraphQLError(error.message.clone()));
            }

            let user_response = response.data.ok_or(GitHubApiError::GraphQLError(
                "No data in response".to_string(),
            ))?;
            let user = user_response.user.ok_or(GitHubApiError::UserNotFound)?;

            all_repos.extend(user.repositories.nodes);
            has_next_page = user.repositories.page_info.has_next_page;
            after_cursor = user.repositories.page_info.end_cursor;
        }

        // Create a set for quick lookup of excluded repositories
        let exclude_set: std::collections::HashSet<&String> = exclude_repos.iter().collect();

        // Create LangEdge structs using the existing pattern
        let mut edges = Vec::new();

        for repo in all_repos {
            // Skip excluded repositories
            if exclude_set.contains(&repo.name) {
                continue;
            }

            // Process language edges for this repository
            for edge in repo.languages.edges {
                edges.push(crate::cards::langs_card::LangEdge {
                    name: edge.node.name,
                    size_bytes: edge.size,
                });
            }
        }

        // Use the existing from_edges method to convert to LanguageStat
        let stats = crate::cards::langs_card::LanguageStat::from_edges(edges);

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_check_with_no_data() {
        let rate_limit = GitHubRateLimit::default();
        let result = check_rate_limit_with_data(&rate_limit);
        assert!(
            result.is_ok(),
            "Should allow request when no rate limit data available"
        );
    }

    #[test]
    fn test_rate_limit_check_with_sufficient_remaining() {
        let rate_limit = GitHubRateLimit {
            limit: Some(5000),
            remaining: Some(500),
            used: Some(4500),
            reset: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 3600, // 1 hour from now
            ),
        };

        let result = check_rate_limit_with_data(&rate_limit);
        assert!(
            result.is_ok(),
            "Should allow request when sufficient requests remaining"
        );
    }

    #[test]
    fn test_rate_limit_protection_triggered() {
        let reset_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // 1 hour from now

        let rate_limit = GitHubRateLimit {
            limit: Some(5000),
            remaining: Some(50), // Below threshold
            used: Some(4950),
            reset: Some(reset_time),
        };

        let result = check_rate_limit_with_data(&rate_limit);
        assert!(
            result.is_err(),
            "Should block request when remaining requests below threshold"
        );

        if let Err(GitHubApiError::RateLimitProtection(remaining, reset)) = result {
            assert_eq!(remaining, 50);
            assert_eq!(reset, reset_time);
        } else {
            panic!("Expected RateLimitProtection error");
        }
    }

    #[test]
    fn test_rate_limit_allows_after_reset_time() {
        let past_reset_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 3600; // 1 hour ago

        let rate_limit = GitHubRateLimit {
            limit: Some(5000),
            remaining: Some(50), // Below threshold
            used: Some(4950),
            reset: Some(past_reset_time),
        };

        let result = check_rate_limit_with_data(&rate_limit);
        assert!(
            result.is_ok(),
            "Should allow request when reset time has passed even with low remaining count"
        );
    }

    #[test]
    fn test_rate_limit_with_partial_data() {
        // Test with only remaining but no reset time
        let rate_limit = GitHubRateLimit {
            limit: Some(5000),
            remaining: Some(50),
            used: Some(4950),
            reset: None, // No reset time
        };

        let result = check_rate_limit_with_data(&rate_limit);
        assert!(
            result.is_ok(),
            "Should allow request when reset time is not available"
        );

        // Test with only reset time but no remaining
        let rate_limit = GitHubRateLimit {
            limit: Some(5000),
            remaining: None, // No remaining count
            used: Some(4950),
            reset: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 3600,
            ),
        };

        let result = check_rate_limit_with_data(&rate_limit);
        assert!(
            result.is_ok(),
            "Should allow request when remaining count is not available"
        );
    }

    #[test]
    fn test_rate_limit_boundary_conditions() {
        let future_reset_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // 1 hour from now

        // Test exactly at threshold (100)
        let rate_limit = GitHubRateLimit {
            limit: Some(5000),
            remaining: Some(100), // Exactly at threshold
            used: Some(4900),
            reset: Some(future_reset_time),
        };

        let result = check_rate_limit_with_data(&rate_limit);
        assert!(
            result.is_ok(),
            "Should allow request when remaining is exactly at threshold"
        );

        // Test just below threshold (99)
        let rate_limit = GitHubRateLimit {
            limit: Some(5000),
            remaining: Some(99), // Below threshold
            used: Some(4901),
            reset: Some(future_reset_time),
        };

        let result = check_rate_limit_with_data(&rate_limit);
        assert!(
            result.is_err(),
            "Should block request when remaining is below threshold"
        );
    }
}
