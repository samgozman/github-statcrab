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

#[tokio::test]
async fn test_health_endpoint_returns_200() {
    let app = app();
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Check for JSON content type
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(content_type.contains("application/json"));

    // Parse JSON response
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check status field
    assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("OK"));

    // Check app version field
    let version = json
        .get("app_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(!version.is_empty(), "app_version should be present");
    assert!(
        version.chars().any(|c| c.is_ascii_digit()),
        "Version should contain digits"
    );

    println!(
        "✓ Health endpoint returns 200 OK with JSON response, version: {}",
        version
    );
}

#[tokio::test]
async fn test_health_endpoint_after_github_api_call() {
    common::setup_integration_test();

    let app = app();
    let username = common::get_test_username();

    // First make a GitHub API call to populate rate limit info
    let stats_req = Request::builder()
        .uri(format!("/stats-card?username={}", username))
        .body(Body::empty())
        .unwrap();

    let stats_resp = app.clone().oneshot(stats_req).await.unwrap();

    // Don't check if the stats call succeeded (might fail due to rate limits, etc.)
    // Just make sure it attempted the GitHub API call
    println!("Stats card response status: {}", stats_resp.status());

    // Now check the health endpoint to see if it has rate limit headers
    let health_req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let health_resp = app.oneshot(health_req).await.unwrap();
    assert_eq!(health_resp.status(), StatusCode::OK);

    // Check for JSON content type
    let content_type = health_resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(content_type.contains("application/json"));

    // Parse JSON response
    let body = health_resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check status field
    assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("OK"));

    // Check app version field (should always be present)
    let version = json
        .get("app_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(!version.is_empty(), "app_version should be present");
    assert!(
        version.chars().any(|c| c.is_ascii_digit()),
        "Version should contain digits"
    );
    println!("✓ App version found: {}", version);

    // Check cache statistics (should always be present)
    let cache = json.get("cache").expect("cache object should be present");

    assert!(
        cache.get("total_entries").is_some(),
        "total_entries should be present"
    );
    assert!(
        cache.get("total_size_bytes").is_some(),
        "total_size_bytes should be present"
    );
    assert!(
        cache.get("stats_entries").is_some(),
        "stats_entries should be present"
    );
    assert!(
        cache.get("stats_size_bytes").is_some(),
        "stats_size_bytes should be present"
    );
    assert!(
        cache.get("languages_entries").is_some(),
        "languages_entries should be present"
    );
    assert!(
        cache.get("languages_size_bytes").is_some(),
        "languages_size_bytes should be present"
    );

    println!("✓ All cache statistics present:");
    if let Some(total_entries) = cache.get("total_entries") {
        println!("  Total entries: {}", total_entries);
    }
    if let Some(total_size) = cache.get("total_size_bytes") {
        println!("  Total size: {}", total_size);
    }
    if let Some(stats_entries) = cache.get("stats_entries") {
        println!("  Stats entries: {}", stats_entries);
    }
    if let Some(stats_size) = cache.get("stats_size_bytes") {
        println!("  Stats size: {}", stats_size);
    }
    if let Some(languages_entries) = cache.get("languages_entries") {
        println!("  Languages entries: {}", languages_entries);
    }
    if let Some(languages_size) = cache.get("languages_size_bytes") {
        println!("  Languages size: {}", languages_size);
    }

    // Check for GitHub rate limit data
    let github_ratelimit = json
        .get("github_ratelimit")
        .expect("github_ratelimit object should be present");

    let has_limit = github_ratelimit.get("limit").is_some();
    let has_remaining = github_ratelimit.get("remaining").is_some();
    let has_used = github_ratelimit.get("used").is_some();
    let has_reset = github_ratelimit.get("reset").is_some();

    println!("GitHub Rate Limit Data Present:");
    println!("  limit: {}", has_limit);
    println!("  remaining: {}", has_remaining);
    println!("  used: {}", has_used);
    println!("  reset: {}", has_reset);

    // If any data is present, it means our rate limit tracking is working
    if has_limit || has_remaining || has_used || has_reset {
        println!("✓ Rate limit tracking is working - data found in health endpoint");

        // Print actual values for debugging
        if let Some(limit) = github_ratelimit.get("limit") {
            println!("  Limit: {}", limit);
        }
        if let Some(remaining) = github_ratelimit.get("remaining") {
            println!("  Remaining: {}", remaining);
        }
        if let Some(used) = github_ratelimit.get("used") {
            println!("  Used: {}", used);
        }
        if let Some(reset) = github_ratelimit.get("reset") {
            println!("  Reset: {}", reset);
        }
    } else {
        println!("⚠️  No rate limit data found (GitHub API call might not have succeeded)");
    }
}
