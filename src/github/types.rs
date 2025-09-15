use serde::{Deserialize, Serialize};

/// GitHub user statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubStats {
    pub name: Option<String>,
    pub login: String,
    pub total_stars: u32,
    pub total_commits_ytd: u32,
    pub total_prs: u32,
    pub total_merged_prs: u32,
    pub total_reviews: u32,
    pub total_issues: u32,
    pub total_discussions_started: u32,
    pub total_discussions_answered: u32,
}

impl GitHubStats {
    /// Create a StatsCard from GitHub statistics
    pub fn to_stats_card(
        &self,
        username: String,
        card_settings: crate::cards::card::CardSettings,
    ) -> crate::cards::stats_card::StatsCard {
        use crate::cards::stats_card::StatsCard;

        StatsCard {
            card_settings,
            username,
            stars_count: Some(self.total_stars),
            commits_ytd_count: Some(self.total_commits_ytd),
            issues_count: Some(self.total_issues),
            pull_requests_count: Some(self.total_prs),
            merge_requests_count: Some(self.total_merged_prs),
            reviews_count: Some(self.total_reviews),
            started_discussions_count: Some(self.total_discussions_started),
            answered_discussions_count: Some(self.total_discussions_answered),
        }
    }
}

/// GitHub API error types
#[derive(thiserror::Error, Debug)]
pub enum GitHubApiError {
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid username: {0}")]
    InvalidUsername(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("GraphQL error: {0}")]
    GraphQLError(String),
    #[error("Missing GitHub token")]
    MissingToken,
}

/// GraphQL response wrapper
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

/// GraphQL query response structures
#[derive(Debug, Deserialize)]
pub struct UserQueryResponse {
    pub user: Option<UserData>,
}

#[derive(Debug, Deserialize)]
pub struct LanguagesQueryResponse {
    pub user: Option<LanguagesUserData>,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub name: Option<String>,
    pub login: String,
    #[serde(rename = "contributionsCollection")]
    pub contributions_collection: ContributionsCollection,
    #[serde(rename = "pullRequests")]
    pub pull_requests: CountableConnection,
    #[serde(rename = "mergedPullRequests")]
    pub merged_pull_requests: Option<CountableConnection>,
    #[serde(rename = "openIssues")]
    pub open_issues: CountableConnection,
    #[serde(rename = "closedIssues")]
    pub closed_issues: CountableConnection,
    #[serde(rename = "repositoryDiscussions")]
    pub repository_discussions: Option<CountableConnection>,
    #[serde(rename = "repositoryDiscussionComments")]
    pub repository_discussion_comments: Option<CountableConnection>,
    pub repositories: RepositoriesConnection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContributionsCollection {
    #[serde(rename = "totalCommitContributions")]
    pub total_commit_contributions: u32,
    #[serde(rename = "totalPullRequestReviewContributions")]
    pub total_pull_request_review_contributions: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CountableConnection {
    #[serde(rename = "totalCount")]
    pub total_count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoriesConnection {
    pub nodes: Vec<RepositoryNode>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryNode {
    pub stargazers: CountableConnection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "endCursor")]
    pub end_cursor: Option<String>,
}

// Language-specific types
#[derive(Debug, Deserialize)]
pub struct LanguagesUserData {
    pub repositories: LanguageRepositoriesConnection,
}

#[derive(Debug, Deserialize)]
pub struct LanguageRepositoriesConnection {
    pub nodes: Vec<LanguageRepositoryNode>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
pub struct LanguageRepositoryNode {
    pub name: String,
    pub languages: LanguagesConnection,
}

#[derive(Debug, Deserialize)]
pub struct LanguagesConnection {
    pub edges: Vec<LanguageEdge>,
}

#[derive(Debug, Deserialize)]
pub struct LanguageEdge {
    pub size: usize,
    pub node: LanguageNode,
}

#[derive(Debug, Deserialize)]
pub struct LanguageNode {
    pub name: String,
}
