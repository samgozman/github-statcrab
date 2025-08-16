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
use crate::cards::langs_card::{LangsCard, LanguageStat, LayoutType};
use crate::cards::stats_card::StatsCard;

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

    // Default values (demo)
    let mut stars_count = Some(2400);
    let mut commits_ytd_count = Some(123);
    let mut issues_count = Some(123);
    let mut pull_requests_count = Some(123);
    let mut merge_requests_count = Some(123);
    let mut reviews_count = Some(123);
    let mut started_discussions_count = Some(123);
    let mut answered_discussions_count = Some(123);

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
                HideStat::StarsCount => stars_count = None,
                HideStat::CommitsYtdCount => commits_ytd_count = None,
                HideStat::IssuesCount => issues_count = None,
                HideStat::PullRequestsCount => pull_requests_count = None,
                HideStat::MergeRequestsCount => merge_requests_count = None,
                HideStat::ReviewsCount => reviews_count = None,
                HideStat::StartedDiscussionsCount => started_discussions_count = None,
                HideStat::AnsweredDiscussionsCount => answered_discussions_count = None,
            }
        }
    }

    // Ensure at least two visible stats remain
    let visible = [
        &stars_count,
        &commits_ytd_count,
        &issues_count,
        &pull_requests_count,
        &merge_requests_count,
        &reviews_count,
        &started_discussions_count,
        &answered_discussions_count,
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

    let svg = StatsCard {
        card_settings: settings,
        username: q.username,
        stars_count,
        commits_ytd_count,
        issues_count,
        pull_requests_count,
        merge_requests_count,
        reviews_count,
        started_discussions_count,
        answered_discussions_count,
    }
    .render();

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

    let stats_stub = vec![
        LanguageStat {
            name: "Rust".to_string(),
            size_bytes: 1000,
            repo_count: 10,
        },
        LanguageStat {
            name: "Go".to_string(),
            size_bytes: 2000,
            repo_count: 5,
        },
        LanguageStat {
            name: "JavaScript".to_string(),
            size_bytes: 1300,
            repo_count: 8,
        },
    ];

    let svg = LangsCard {
        card_settings: settings,
        layout: q.layout.unwrap_or(LayoutTypeQuery::Horizontal).into(),
        stats: stats_stub,
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
    #[serde(rename = "horizontal")]
    Horizontal,
    #[serde(rename = "vertical")]
    Vertical,
}

impl From<LayoutTypeQuery> for LayoutType {
    fn from(layout: LayoutTypeQuery) -> Self {
        match layout {
            LayoutTypeQuery::Horizontal => LayoutType::Horizontal,
            LayoutTypeQuery::Vertical => LayoutType::Vertical,
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

        #[tokio::test]
        async fn ok_with_username_and_returns_svg() {
            let app = app();
            let username = "alice";
            let req = Request::builder()
                .uri(format!("/stats-card?username={username}"))
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert_eq!(content_type, "image/svg+xml");

            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert!(body_str.contains("<svg"));
            assert!(body_str.contains(&format!("@{username}: GitHub Stats")));
        }

        #[tokio::test]
        async fn invalid_hide_value_returns_400() {
            let app = app();
            let req = Request::builder()
                .uri("/stats-card?username=alice&hide=foo")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let msg = v.get("error").and_then(|v| v.as_str()).unwrap_or("");
            assert!(msg.contains("invalid hide value"));
        }

        #[tokio::test]
        async fn hiding_too_many_stats_returns_400() {
            // Hide 7 out of 8 stats so only one remains visible
            let hide = [
                "stars_count",
                "commits_ytd_count",
                "issues_count",
                "pull_requests_count",
                "merge_requests_count",
                "reviews_count",
                "started_discussions_count",
                // leave answered_discussions_count visible
            ]
            .join(",");

            let app = app();
            let req = Request::builder()
                .uri(format!("/stats-card?username=alice&hide={hide}"))
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let msg = v.get("error").and_then(|v| v.as_str()).unwrap_or("");
            assert!(msg.contains("at least 2 must remain"));
        }

        #[tokio::test]
        async fn ok_hide_subset_removes_labels_from_svg() {
            let app = app();
            // Hide stars and pull requests
            let req = Request::builder()
                .uri("/stats-card?username=alice&hide=stars_count,pull_requests_count")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert!(!body_str.contains("Stars:"));
            assert!(!body_str.contains("Pull Requests:"));
            // Some other stat should still be present
            assert!(body_str.contains("Issues:"));
        }

        #[tokio::test]
        async fn ok_with_theme_query_param() {
            let app = app();
            let req = Request::builder()
                .uri("/stats-card?username=alice&theme=transparent_blue")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert_eq!(content_type, "image/svg+xml");
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert!(body_str.contains("<svg"));
        }
        #[tokio::test]
        async fn with_unknown_theme_returns_400() {
            let app = app();
            let req = Request::builder()
                .uri("/stats-card?username=alice&theme=unknown_theme")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap_or_default();
            assert!(body_str.contains("unknown variant `unknown_theme`"));
        }

        #[tokio::test]
        async fn hide_title_param_hides_title_group() {
            let app = app();
            let req = Request::builder()
                .uri("/stats-card?username=alice&hide_title=true")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            // <title> tag should still exist for accessibility
            assert!(body_str.contains("<title id=\"title-id\">@alice: GitHub Stats</title>"));
            // Visual title group should be absent
            assert!(!body_str.contains("class=\"title\""));
        }

        #[tokio::test]
        async fn hide_background_param_hides_background_rect() {
            let app = app();
            let req = Request::builder()
                .uri("/stats-card?username=alice&hide_background=true")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert!(!body_str.contains("<rect class=\"background\""));
        }

        #[tokio::test]
        async fn hide_background_stroke_param_hides_stroke_only() {
            let app = app();
            let req = Request::builder()
                .uri("/stats-card?username=alice&hide_background_stroke=true")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            // Background still present
            assert!(body_str.contains("<rect class=\"background\""));
            // Stroke opacity hidden
            assert!(body_str.contains("stroke-opacity=\"0\""));
        }

        #[tokio::test]
        async fn offsets_affect_title_translation() {
            let app = app();
            // Card::TITLE_FONT_SIZE=18 so translate(x, 18 + offset_y)
            let req = Request::builder()
                .uri("/stats-card?username=alice&offset_x=30&offset_y=25")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert!(body_str.contains("translate(30, 43)"));
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
        async fn ok_with_username_and_returns_svg() {
            let app = app();
            let req = Request::builder()
                .uri("/langs-card?username=alice")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert_eq!(content_type, "image/svg+xml");
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert!(body_str.contains("<svg"));
            assert!(body_str.contains("Most used languages"));
            // All three stub languages should be present by default
            assert!(body_str.contains(">Go</text>"));
            assert!(body_str.contains(">JavaScript</text>"));
            assert!(body_str.contains(">Rust</text>"));
            // Percentages are rounded first then formatted => 47,30,23
            assert!(body_str.contains("47.00%"));
            assert!(body_str.contains("30.00%"));
            assert!(body_str.contains("23.00%"));
        }

        #[tokio::test]
        async fn max_languages_limits_rows() {
            let app = app();
            let req = Request::builder()
                .uri("/langs-card?username=alice&max_languages=2")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            // Only two rows should be rendered
            assert_eq!(body_str.matches("<g class=\"row\">").count(), 2);
            assert!(body_str.contains(">Go</text>"));
            assert!(body_str.contains(">JavaScript</text>"));
            // Rust should be excluded
            assert!(!body_str.contains(">Rust</text>"));
        }

        #[tokio::test]
        async fn size_and_count_weights_affect_order_and_percentages() {
            let app = app();
            // size_weight=0, count_weight=1 ranks by repo_count -> Rust(10), JavaScript(8), Go(5)
            let req = Request::builder()
                .uri("/langs-card?username=alice&size_weight=0&count_weight=1")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            // Percentages (counts 10,8,5 / 23 => 43,35,22 after rounding)
            assert!(body_str.contains("43.00%"));
            assert!(body_str.contains("35.00%"));
            assert!(body_str.contains("22.00%"));
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
        async fn hide_title_param_hides_title_group() {
            let app = app();
            let req = Request::builder()
                .uri("/langs-card?username=alice&hide_title=true")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            // Title tag remains with static title
            assert!(body_str.contains("<title id=\"title-id\">Most used languages</title>"));
            // Visual title group should be absent
            assert!(!body_str.contains("class=\"title\""));
        }

        #[tokio::test]
        async fn hide_background_param_hides_background_rect() {
            let app = app();
            let req = Request::builder()
                .uri("/langs-card?username=alice&hide_background=true")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert!(!body_str.contains("<rect class=\"background\""));
        }

        #[tokio::test]
        async fn hide_background_stroke_param_hides_stroke_only() {
            let app = app();
            let req = Request::builder()
                .uri("/langs-card?username=alice&hide_background_stroke=true")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert!(body_str.contains("<rect class=\"background\""));
            assert!(body_str.contains("stroke-opacity=\"0\""));
        }

        #[tokio::test]
        async fn offsets_affect_title_translation() {
            let app = app();
            let req = Request::builder()
                .uri("/langs-card?username=alice&offset_x=28&offset_y=10")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            // Card::TITLE_FONT_SIZE=18 -> translate(28, 28)
            assert!(body_str.contains("translate(28, 28)"));
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
