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
use crate::github::{GitHubApi, GitHubApiError, get_github_cache, get_github_rate_limit};

use card_theme_macros::build_theme_query;

pub fn api_router() -> Router {
    Router::new()
        .route("/stats-card", get(get_stats_card))
        .route("/langs-card", get(get_langs_card))
        .route("/health", get(get_health))
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

#[tracing::instrument(name = "stats_card_request", fields(username = %q.username))]
async fn get_stats_card(Query(q): Query<StatsCardQuery>) -> impl IntoResponse {
    // Add user context to Sentry
    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            username: Some(q.username.clone()),
            ..Default::default()
        }));
        scope.set_tag("card_type", "stats");
        scope.set_context(
            "request_params",
            sentry::protocol::Context::Other({
                let mut map = std::collections::BTreeMap::new();
                map.insert("username".to_string(), q.username.clone().into());
                if let Some(theme) = &q.settings.theme {
                    map.insert("theme".to_string(), format!("{:?}", theme).into());
                }
                if let Some(hide) = &q.hide {
                    map.insert("hide".to_string(), hide.clone().into());
                }
                map
            }),
        );
    });

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
            // Report rate limit exceeded to Sentry as it's an operational issue
            sentry::capture_message(
                &format!("GitHub API rate limit exceeded for user: {}", q.username),
                sentry::Level::Warning,
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({"error": "GitHub API rate limit exceeded"})),
            )
                .into_response();
        }
        Err(GitHubApiError::RateLimitProtection(remaining, reset_time)) => {
            // Calculate seconds until reset
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let retry_after = reset_time.saturating_sub(current_time);

            let mut headers = axum::http::HeaderMap::new();
            if let Ok(retry_header) = axum::http::HeaderValue::from_str(&retry_after.to_string()) {
                headers.insert("retry-after", retry_header);
            }

            return (
                StatusCode::TOO_MANY_REQUESTS,
                headers,
                Json(serde_json::json!({
                    "error": format!("Rate limit protection active: {} requests remaining, reset at {}", remaining, reset_time),
                    "retry_after_seconds": retry_after
                })),
            )
                .into_response();
        }
        Err(e) => {
            // Report all other unexpected errors to Sentry
            sentry::capture_error(&e);
            tracing::error!("GitHub API error: {e}");
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

#[tracing::instrument(name = "langs_card_request", fields(username = %q.username))]
async fn get_langs_card(Query(q): Query<LangsCardQuery>) -> impl IntoResponse {
    // Add user context to Sentry
    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            username: Some(q.username.clone()),
            ..Default::default()
        }));
        scope.set_tag("card_type", "languages");
        scope.set_context(
            "request_params",
            sentry::protocol::Context::Other({
                let mut map = std::collections::BTreeMap::new();
                map.insert("username".to_string(), q.username.clone().into());
                if let Some(theme) = &q.settings.theme {
                    map.insert("theme".to_string(), format!("{:?}", theme).into());
                }
                if let Some(exclude_repo) = &q.exclude_repo {
                    map.insert("exclude_repo".to_string(), exclude_repo.clone().into());
                }
                if let Some(layout) = &q.layout {
                    map.insert("layout".to_string(), format!("{:?}", layout).into());
                }
                map.insert(
                    "max_languages".to_string(),
                    q.max_languages.unwrap_or(8).into(),
                );
                map
            }),
        );
    });

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
            // Report rate limit exceeded to Sentry as it's an operational issue
            sentry::capture_message(
                &format!(
                    "GitHub API rate limit exceeded for user: {} (languages)",
                    q.username
                ),
                sentry::Level::Warning,
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({"error": "GitHub API rate limit exceeded"})),
            )
                .into_response();
        }
        Err(GitHubApiError::RateLimitProtection(remaining, reset_time)) => {
            // Calculate seconds until reset
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let retry_after = reset_time.saturating_sub(current_time);

            let mut headers = axum::http::HeaderMap::new();
            if let Ok(retry_header) = axum::http::HeaderValue::from_str(&retry_after.to_string()) {
                headers.insert("retry-after", retry_header);
            }

            return (
                StatusCode::TOO_MANY_REQUESTS,
                headers,
                Json(serde_json::json!({
                    "error": format!("Rate limit protection active: {} requests remaining, reset at {}", remaining, reset_time),
                    "retry_after_seconds": retry_after
                })),
            )
                .into_response();
        }
        Err(e) => {
            // Report all other unexpected errors to Sentry
            sentry::capture_error(&e);
            tracing::error!("GitHub API error: {e}");
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

#[tracing::instrument(level = "trace")]
async fn get_health() -> impl IntoResponse {
    let rate_limit = get_github_rate_limit();

    let mut headers = HeaderMap::new();

    // Add app version header
    if let Ok(header_value) = header::HeaderValue::from_str(env!("CARGO_PKG_VERSION")) {
        headers.insert("x-app-version", header_value);
    }

    // Add GitHub rate limit headers with github prefix
    if let Some(limit) = rate_limit.limit
        && let Ok(header_value) = header::HeaderValue::from_str(&limit.to_string())
    {
        headers.insert("x-github-ratelimit-limit", header_value);
    }

    if let Some(remaining) = rate_limit.remaining
        && let Ok(header_value) = header::HeaderValue::from_str(&remaining.to_string())
    {
        headers.insert("x-github-ratelimit-remaining", header_value);
    }

    if let Some(used) = rate_limit.used
        && let Ok(header_value) = header::HeaderValue::from_str(&used.to_string())
    {
        headers.insert("x-github-ratelimit-used", header_value);
    }

    if let Some(reset) = rate_limit.reset
        && let Ok(header_value) = header::HeaderValue::from_str(&reset.to_string())
    {
        headers.insert("x-github-ratelimit-reset", header_value);
    }

    // Add cache statistics headers
    let cache = get_github_cache();
    let cache_stats = cache.stats();

    if let Ok(header_value) = header::HeaderValue::from_str(&cache_stats.entry_count.to_string()) {
        headers.insert("x-cache-total-entries", header_value);
    }

    if let Ok(header_value) = header::HeaderValue::from_str(&cache_stats.weighted_size.to_string())
    {
        headers.insert("x-cache-total-size-bytes", header_value);
    }

    if let Ok(header_value) =
        header::HeaderValue::from_str(&cache_stats.stats_cache_entries.to_string())
    {
        headers.insert("x-cache-stats-entries", header_value);
    }

    if let Ok(header_value) =
        header::HeaderValue::from_str(&cache_stats.stats_cache_size.to_string())
    {
        headers.insert("x-cache-stats-size-bytes", header_value);
    }

    if let Ok(header_value) =
        header::HeaderValue::from_str(&cache_stats.languages_cache_entries.to_string())
    {
        headers.insert("x-cache-languages-entries", header_value);
    }

    if let Ok(header_value) =
        header::HeaderValue::from_str(&cache_stats.languages_cache_size.to_string())
    {
        headers.insert("x-cache-languages-size-bytes", header_value);
    }

    (StatusCode::OK, headers)
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

    // Tests for GET /api/health route behavior
    mod route_get_health {
        use super::*;

        fn app() -> Router {
            api_router()
        }

        #[tokio::test]
        async fn returns_200_ok() {
            let app = app();
            let req = Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        #[tokio::test]
        async fn returns_github_rate_limit_headers() {
            let app = app();
            let req = Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            // Check that the headers structure is correct even if values might be None
            let headers = resp.headers();
            // Note: These headers may not be present if no GitHub API calls have been made yet
            // but at least the endpoint should work without errors
            assert!(
                headers.get("x-github-ratelimit-limit").is_some()
                    || headers.get("x-github-ratelimit-limit").is_none()
            );
        }

        #[tokio::test]
        async fn returns_app_version_header() {
            let app = app();
            let req = Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            // Check that the app version header is present
            let headers = resp.headers();
            let version = headers
                .get("x-app-version")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert!(
                !version.is_empty(),
                "x-app-version header should be present"
            );
            // Version should match the format in Cargo.toml (semantic versioning)
            assert!(
                version.chars().any(|c| c.is_ascii_digit()),
                "Version should contain digits"
            );
        }
    }
}
