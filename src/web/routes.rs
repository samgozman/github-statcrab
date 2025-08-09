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
use crate::cards::stats_card::StatsCard;

pub fn api_router() -> Router {
    Router::new().route("/stats-card", get(get_stats_card))
}

#[derive(Debug, Deserialize)]
pub enum ThemeQuery {
    TransparentBlue,
}

impl From<ThemeQuery> for CardTheme {
    fn from(t: ThemeQuery) -> Self {
        match t {
            ThemeQuery::TransparentBlue => CardTheme::TransparentBlue,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StatsCardQuery {
    // required
    username: String,
    // common optional visuals
    offset_x: Option<u32>,
    offset_y: Option<u32>,
    theme: Option<ThemeQuery>,
    hide_title: Option<bool>,
    hide_background: Option<bool>,
    hide_background_stroke: Option<bool>,
    // comma-separated array: e.g. ?hide=stars_count,commits_ytd_count
    hide: Option<String>,
}

fn svg_response(svg: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("image/svg+xml"),
    );
    (StatusCode::OK, headers, svg).into_response()
}

async fn get_stats_card(Query(q): Query<StatsCardQuery>) -> impl IntoResponse {
    if q.username.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error":"username is required"})),
        )
            .into_response();
    }

    // Base settings from Default
    let mut settings = CardSettings {
        offset_x: 12,
        offset_y: 12,
        theme: CardTheme::TransparentBlue,
        hide_title: false,
        hide_background: false,
        hide_background_stroke: false,
    };

    if let Some(x) = q.offset_x {
        settings.offset_x = x;
    }
    if let Some(y) = q.offset_y {
        settings.offset_y = y;
    }
    if let Some(t) = q.theme {
        settings.theme = t.into();
    }
    if let Some(v) = q.hide_title {
        settings.hide_title = v;
    }
    if let Some(v) = q.hide_background {
        settings.hide_background = v;
    }
    if let Some(v) = q.hide_background_stroke {
        settings.hide_background_stroke = v;
    }

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
        async fn hide_subset_removes_labels_from_svg() {
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
    }
}
