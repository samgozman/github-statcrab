//! Web Routes Integration Tests
//!
//! These tests require a real GitHub API token to be set in the GITHUB_TOKEN environment variable.
//! They test the actual web endpoints with real GitHub API calls.
//!
//! Run with: cargo test --test web_routes_integration

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use github_statcrab::web::routes::api_router;
use http_body_util::BodyExt as _; // for collect()
use tower::ServiceExt; // for oneshot()

mod common;

fn app() -> axum::Router {
    api_router()
}

#[tokio::test]
async fn test_stats_card_with_real_user() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();
    let req = Request::builder()
        .uri(&format!("/stats-card?username={}", username))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(content_type, "image/svg+xml");

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify it's valid SVG
    assert!(body_str.contains("<svg"));
    assert!(body_str.contains("</svg>"));

    // Verify it contains the username
    assert!(body_str.contains(&format!("@{}", username)));

    println!("✓ Successfully generated stats card for user: {}", username);
}

#[tokio::test]
async fn test_stats_card_with_nonexistent_user() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_invalid_username();
    let req = Request::builder()
        .uri(&format!("/stats-card?username={}", username))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let error_msg = json.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(error_msg.contains("User not found"));

    println!("✓ Correctly handled nonexistent user: {}", username);
}

#[tokio::test]
async fn test_stats_card_with_hide_params() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();

    // Hide stars and pull requests
    let req = Request::builder()
        .uri(&format!(
            "/stats-card?username={}&hide=stars_count,pull_requests_count",
            username
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify hidden stats are not present
    assert!(!body_str.contains("Stars:"));
    assert!(!body_str.contains("Pull Requests:"));

    // Some other stats should still be present
    assert!(
        body_str.contains("Issues:")
            || body_str.contains("Commits:")
            || body_str.contains("Reviews:")
    );

    println!("✓ Successfully hid stats for user: {}", username);
}

#[tokio::test]
async fn test_stats_card_with_theme() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();
    let req = Request::builder()
        .uri(&format!(
            "/stats-card?username={}&theme=transparent_blue",
            username
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(content_type, "image/svg+xml");

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("<svg"));

    println!(
        "✓ Successfully generated themed stats card for user: {}",
        username
    );
}

#[tokio::test]
async fn test_stats_card_with_hide_visual_options() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();

    // Test hide_title
    let req = Request::builder()
        .uri(&format!(
            "/stats-card?username={}&hide_title=true",
            username
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Title should still exist for accessibility but visual title group should be absent
    assert!(body_str.contains(&format!("@{}: GitHub Stats", username)));
    assert!(!body_str.contains("class=\"title\""));

    println!("✓ Successfully hid title for user: {}", username);
}

#[tokio::test]
async fn test_invalid_hide_value() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();
    let req = Request::builder()
        .uri(&format!(
            "/stats-card?username={}&hide=invalid_stat",
            username
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let error_msg = json.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(error_msg.contains("invalid hide value"));

    println!("✓ Correctly rejected invalid hide value");
}

#[tokio::test]
async fn test_hiding_too_many_stats() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();

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

    let req = Request::builder()
        .uri(&format!("/stats-card?username={}&hide={}", username, hide))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let error_msg = json.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(error_msg.contains("at least 2 must remain"));

    println!("✓ Correctly prevented hiding too many stats");
}
