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
        .uri(format!("/stats-card?username={}", username))
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
        .uri(format!("/stats-card?username={}", username))
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
        .uri(format!(
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
        .uri(format!(
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
        .uri(format!("/stats-card?username={}&hide_title=true", username))
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
        .uri(format!(
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
        .uri(format!("/stats-card?username={}&hide={}", username, hide))
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

#[tokio::test]
async fn test_langs_card_with_real_user() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();
    let req = Request::builder()
        .uri(format!("/langs-card?username={}", username))
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
    assert!(body_str.contains("Most used languages"));

    // Should contain at least one programming language
    // The specific languages depend on the user's repositories
    let has_language = body_str.contains(">")
        && (body_str.contains("JavaScript")
            || body_str.contains("TypeScript")
            || body_str.contains("Python")
            || body_str.contains("Rust")
            || body_str.contains("Go")
            || body_str.contains("Java")
            || body_str.contains("C++")
            || body_str.contains("C#"));

    assert!(
        has_language,
        "Response should contain at least one programming language"
    );

    println!("✓ Successfully generated langs card for user: {}", username);
}

#[tokio::test]
async fn test_langs_card_with_nonexistent_user() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_invalid_username();
    let req = Request::builder()
        .uri(format!("/langs-card?username={}", username))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let error_msg = json.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(error_msg.contains("User not found"));

    println!(
        "✓ Correctly handled nonexistent user for langs card: {}",
        username
    );
}

#[tokio::test]
async fn test_langs_card_with_horizontal_layout() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();
    let req = Request::builder()
        .uri(format!(
            "/langs-card?username={}&layout=horizontal",
            username
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify it's valid SVG
    assert!(body_str.contains("<svg"));
    assert!(body_str.contains("Most used languages"));

    println!(
        "✓ Successfully generated horizontal langs card for user: {}",
        username
    );
}

#[tokio::test]
async fn test_langs_card_with_max_languages_limit() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();
    let req = Request::builder()
        .uri(format!("/langs-card?username={}&max_languages=3", username))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify it's valid SVG
    assert!(body_str.contains("<svg"));
    assert!(body_str.contains("Most used languages"));

    // Should have at most 3 language rows
    let row_count = body_str.matches("<g class=\"row\">").count();
    assert!(
        row_count <= 3,
        "Should have at most 3 language rows, found: {}",
        row_count
    );

    println!(
        "✓ Successfully limited languages to max 3 for user: {}",
        username
    );
}

#[tokio::test]
async fn test_langs_card_with_size_and_count_weights() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();

    // Test with size_weight=0 and count_weight=1 (rank by repository count only)
    let req = Request::builder()
        .uri(format!(
            "/langs-card?username={}&size_weight=0&count_weight=1",
            username
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify it's valid SVG
    assert!(body_str.contains("<svg"));
    assert!(body_str.contains("Most used languages"));

    // Should contain percentage values with decimal places (not just 0.00%)
    let has_non_zero_percentage = body_str
        .lines()
        .any(|line| line.contains("%") && !line.contains("0.00%"));

    assert!(
        has_non_zero_percentage,
        "Should contain non-zero percentages"
    );

    println!(
        "✓ Successfully generated weighted langs card for user: {}",
        username
    );
}

#[tokio::test]
async fn test_langs_card_with_exclude_repositories() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();

    // Exclude some repositories (using common repo names that might exist)
    let req = Request::builder()
        .uri(format!(
            "/langs-card?username={}&exclude_repo=dotfiles,config",
            username
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify it's valid SVG
    assert!(body_str.contains("<svg"));
    assert!(body_str.contains("Most used languages"));

    println!(
        "✓ Successfully generated langs card with excluded repos for user: {}",
        username
    );
}

#[tokio::test]
async fn test_langs_card_with_theme() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();
    let req = Request::builder()
        .uri(format!(
            "/langs-card?username={}&theme=transparent_blue",
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
    assert!(body_str.contains("Most used languages"));

    println!(
        "✓ Successfully generated themed langs card for user: {}",
        username
    );
}

#[tokio::test]
async fn test_langs_card_with_hide_visual_options() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();

    // Test hide_title
    let req = Request::builder()
        .uri(format!("/langs-card?username={}&hide_title=true", username))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Title should still exist for accessibility but visual title group should be absent
    assert!(body_str.contains("<title id=\"title-id\">Most used languages</title>"));
    assert!(!body_str.contains("class=\"title\""));

    println!(
        "✓ Successfully hid title in langs card for user: {}",
        username
    );
}

#[tokio::test]
async fn test_langs_card_invalid_username() {
    common::setup_integration_test();

    let app = app();
    let req = Request::builder()
        .uri("/langs-card?username=bad%20user")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let error_msg = json.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(error_msg.contains("Username cannot contain spaces"));

    println!("✓ Correctly rejected invalid username in langs card");
}

#[tokio::test]
async fn test_langs_card_with_unknown_theme_returns_400() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();
    let req = Request::builder()
        .uri(format!(
            "/langs-card?username={}&theme=unknown_theme",
            username
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap_or_default();
    assert!(body_str.contains("unknown variant `unknown_theme`"));

    println!("✓ Correctly rejected unknown theme in langs card");
}
