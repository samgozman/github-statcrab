use axum::{
    Json, Router,
    extract::Query,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use std::{collections::HashSet, str::FromStr};

use crate::cards::card::{CardSettings, CardTheme};
use crate::cards::langs_card::{LangsCard, LayoutType};
use crate::github::{GitHubApi, GitHubApiError};

use card_theme_macros::build_theme_query;

pub fn api_router() -> Router {
    Router::new()
        .route("/stats-card", get(get_stats_card))
        .route("/langs-card", get(get_langs_card))
}

#[derive(Debug, Deserialize)]
pub struct StatsCardQuery {
    // required
    username: String,
    // flattened common settings
    #[serde(flatten)]
    settings: CardSettingsQuery,
    // comma-separated array: e.g. ?hide=stars_count,commits_ytd_count
    hide: Option<String>,
}

async fn get_stats_card(Query(q): Query<StatsCardQuery>) -> impl IntoResponse {
    // Validate username
    if let Err(e) = validate_username(&q.username) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
            .into_response();
    }

    // Build card settings from query (with defaults applied)
    let settings = q.settings.into_settings();

    // Create GitHub API client
    let github_api = GitHubApi::new();

    // Fetch real stats from GitHub
    let github_stats = match github_api.fetch_user_stats(&q.username).await {
        Ok(stats) => stats,
        Err(GitHubApiError::UserNotFound) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "User not found"})),
            )
                .into_response();
        }
        Err(GitHubApiError::InvalidUsername(msg)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": msg})),
            )
                .into_response();
        }
        Err(GitHubApiError::MissingToken) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "GitHub API token not configured"})),
            )
                .into_response();
        }
        Err(GitHubApiError::RateLimitExceeded) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({"error": "GitHub API rate limit exceeded"})),
            )
                .into_response();
        }
        Err(e) => {
            eprintln!("GitHub API error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch user statistics"})),
            )
                .into_response();
        }
    };

    // Create StatsCard directly from GitHub stats
    let mut stats_card = github_stats.to_stats_card(q.username.clone(), settings);

    // Parse and apply hide list
    if let Some(hide_str) = q.hide.as_deref() {
        let mut to_hide: HashSet<HideStat> = HashSet::new();
        if !hide_str.trim().is_empty() {
            for token in hide_str.split(',') {
                let token = token.trim();
                if token.is_empty() {
                    continue;
                }
                match HideStat::from_str(token) {
                    Ok(v) => {
                        to_hide.insert(v);
                    }
                    Err(_) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({"error": format!("invalid hide value: {}", token)})),
                        )
                            .into_response();
                    }
                }
            }
        }

        for h in to_hide {
            match h {
                HideStat::StarsCount => stats_card.stars_count = None,
                HideStat::CommitsYtdCount => stats_card.commits_ytd_count = None,
                HideStat::IssuesCount => stats_card.issues_count = None,
                HideStat::PullRequestsCount => stats_card.pull_requests_count = None,
                HideStat::MergeRequestsCount => stats_card.merge_requests_count = None,
                HideStat::ReviewsCount => stats_card.reviews_count = None,
                HideStat::StartedDiscussionsCount => stats_card.started_discussions_count = None,
                HideStat::AnsweredDiscussionsCount => stats_card.answered_discussions_count = None,
            }
        }
    }

    // Ensure at least two visible stats remain
    let visible = [
        &stats_card.stars_count,
        &stats_card.commits_ytd_count,
        &stats_card.issues_count,
        &stats_card.pull_requests_count,
        &stats_card.merge_requests_count,
        &stats_card.reviews_count,
        &stats_card.started_discussions_count,
        &stats_card.answered_discussions_count,
    ]
    .iter()
    .filter(|v| v.is_some())
    .count();

    if visible < 2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "hide would remove too many stats; at least 2 must remain"})),
        )
            .into_response();
    }

    let svg = stats_card.render();

    svg_response(svg)
}

#[derive(Debug, Deserialize)]
pub struct LangsCardQuery {
    // required
    username: String,
    // flattened common settings
    #[serde(flatten)]
    settings: CardSettingsQuery,
    // optional stats
    layout: Option<LayoutTypeQuery>,
    size_weight: Option<f64>,
    count_weight: Option<f64>,
    max_languages: Option<u64>,
    // comma-separated list of repositories to exclude
    exclude_repo: Option<String>,
}

async fn get_langs_card(Query(q): Query<LangsCardQuery>) -> impl IntoResponse {
    // Validate username
    if let Err(e) = validate_username(&q.username) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
            .into_response();
    }

    // Build card settings from query (with defaults applied)
    let settings = q.settings.into_settings();

    // Parse excluded repositories
    let exclude_repos: Vec<String> = if let Some(exclude_str) = q.exclude_repo.as_deref() {
        exclude_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    // Create GitHub API client
    let github_api = GitHubApi::new();

    // Fetch real language stats from GitHub
    let language_stats = match github_api
        .fetch_user_languages(&q.username, &exclude_repos)
        .await
    {
        Ok(stats) => stats,
        Err(GitHubApiError::UserNotFound) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "User not found"})),
            )
                .into_response();
        }
        Err(GitHubApiError::InvalidUsername(msg)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": msg})),
            )
                .into_response();
        }
        Err(GitHubApiError::MissingToken) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "GitHub API token not configured"})),
            )
                .into_response();
        }
        Err(GitHubApiError::RateLimitExceeded) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({"error": "GitHub API rate limit exceeded"})),
            )
                .into_response();
        }
        Err(e) => {
            eprintln!("GitHub API error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch user languages"})),
            )
                .into_response();
        }
    };

    let svg = LangsCard {
        card_settings: settings,
        layout: q.layout.unwrap_or(LayoutTypeQuery::Vertical).into(),
        stats: language_stats,
        size_weight: q.size_weight,
        count_weight: q.count_weight,
        max_languages: q.max_languages,
    }
    .render();

    svg_response(svg)
}

/// Helper function to create a response with SVG content and appropriate headers
fn svg_response(svg: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("image/svg+xml"),
    );
    (StatusCode::OK, headers, svg).into_response()
}

fn validate_username(username: &str) -> Result<(), String> {
    if username.trim().is_empty() {
        return Err("Username cannot be empty".to_string());
    }
    if username.contains(' ') {
        return Err("Username cannot contain spaces".to_string());
    }
    Ok(())
}

// Build the ThemeQuery enum from the macro
build_theme_query!();

/// Common query parameters for building [CardSettings] reused across card endpoints.
#[derive(Debug, Deserialize)]
struct CardSettingsQuery {
    // common optional visuals
    offset_x: Option<String>,
    offset_y: Option<String>,
    theme: Option<ThemeQuery>,
    hide_title: Option<String>,
    hide_background: Option<String>,
    hide_background_stroke: Option<String>,
}

impl CardSettingsQuery {
    fn into_settings(self) -> CardSettings {
        CardSettings {
            offset_x: self
                .offset_x
                .as_deref()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(12),
            offset_y: self
                .offset_y
                .as_deref()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(12),
            theme: self
                .theme
                .map(|t| t.into())
                .unwrap_or(CardTheme::TransparentBlue),
            hide_title: self
                .hide_title
                .as_deref()
                .map(|s| s == "true")
                .unwrap_or(false),
            hide_background: self
                .hide_background
                .as_deref()
                .map(|s| s == "true")
                .unwrap_or(false),
            hide_background_stroke: self
                .hide_background_stroke
                .as_deref()
                .map(|s| s == "true")
                .unwrap_or(false),
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HideStat {
    StarsCount,
    CommitsYtdCount,
    IssuesCount,
    PullRequestsCount,
    MergeRequestsCount,
    ReviewsCount,
    StartedDiscussionsCount,
    AnsweredDiscussionsCount,
}

impl FromStr for HideStat {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stars_count" => Ok(HideStat::StarsCount),
            "commits_ytd_count" => Ok(HideStat::CommitsYtdCount),
            "issues_count" => Ok(HideStat::IssuesCount),
            "pull_requests_count" => Ok(HideStat::PullRequestsCount),
            "merge_requests_count" => Ok(HideStat::MergeRequestsCount),
            "reviews_count" => Ok(HideStat::ReviewsCount),
            "started_discussions_count" => Ok(HideStat::StartedDiscussionsCount),
            "answered_discussions_count" => Ok(HideStat::AnsweredDiscussionsCount),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Deserialize)]
enum LayoutTypeQuery {
    #[serde(rename = "vertical")]
    Vertical,
    #[serde(rename = "horizontal")]
    Horizontal,
}

impl From<LayoutTypeQuery> for LayoutType {
    fn from(layout: LayoutTypeQuery) -> Self {
        match layout {
            LayoutTypeQuery::Vertical => LayoutType::Vertical,
            LayoutTypeQuery::Horizontal => LayoutType::Horizontal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt as _; // for collect()
    use tower::ServiceExt; // for oneshot()

    // Tests for the helper function that builds an SVG response
    mod fn_svg_response {
        use super::*;

        #[tokio::test]
        async fn returns_svg_with_correct_headers_and_body() {
            let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>".to_string();
            let resp = svg_response(svg.clone());

            assert_eq!(resp.status(), StatusCode::OK);
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert_eq!(content_type, "image/svg+xml");

            let bytes = resp.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(bytes, svg);
        }
    }

    // Tests for GET /api/stats-card route behavior
    mod route_get_stats_card {
        use super::*;

        fn app() -> Router {
            // Reuse only the API router which mounts /stats-card
            api_router()
        }

        #[tokio::test]
        async fn requires_username_param() {
            let app = app();
            let req = Request::builder()
                .uri("/stats-card")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }
    }

    // Tests for GET /api/langs-card route behavior
    mod route_get_langs_card {
        use super::*;

        fn app() -> Router {
            api_router()
        }

        #[tokio::test]
        async fn requires_username_param() {
            let app = app();
            let req = Request::builder()
                .uri("/langs-card")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn invalid_username_returns_400() {
            let app = app();
            let req = Request::builder()
                .uri("/langs-card?username=bad%20user")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn with_unknown_theme_returns_400() {
            let app = app();
            let req = Request::builder()
                .uri("/langs-card?username=alice&theme=unknown_theme")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap_or_default();
            assert!(body_str.contains("unknown variant `unknown_theme`"));
        }
    }
}
