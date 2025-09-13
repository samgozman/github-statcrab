use reqwest::Client;
use serde_json::json;
use std::env;

use crate::github::types::*;

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
                followers {
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
    async fn execute_query<T>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<GraphQLResponse<T>, GitHubApiError>
    where
        T: serde::de::DeserializeOwned,
    {
        let token = self.token.as_ref().ok_or(GitHubApiError::MissingToken)?;

        let payload = json!({
            "query": query,
            "variables": variables
        });

        let response = self
            .client
            .post("https://api.github.com/graphql")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "github-statcrab")
            .json(&payload)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(GitHubApiError::MissingToken);
        }

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(GitHubApiError::RateLimitExceeded);
        }

        let response_body: GraphQLResponse<T> = response.json().await?;
        Ok(response_body)
    }

    /// Fetch user statistics from GitHub
    pub async fn fetch_user_stats(&self, username: &str) -> Result<GitHubStats, GitHubApiError> {
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
            followers: user.followers.total_count,
        };

        Ok(stats)
    }

    /// Fetch user languages from GitHub
    pub async fn fetch_user_languages(
        &self,
        username: &str,
        exclude_repos: &[String],
    ) -> Result<Vec<crate::cards::langs_card::LanguageStat>, GitHubApiError> {
        Self::validate_username(username)?;

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
