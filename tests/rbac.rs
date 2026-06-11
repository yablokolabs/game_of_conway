mod common;

use base64::Engine;
use reqwest::StatusCode;
use serde_json::json;

// ---------------------------------------------------------------------------
// Registration returns role
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_returns_user_role() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();
    let username = app.unique_name("rbac_reg");

    let res = client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&json!({"username": username, "password": "password123"}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["role"], "user", "newly registered user should have 'user' role");
}

// ---------------------------------------------------------------------------
// User can access their own history
// ---------------------------------------------------------------------------

#[tokio::test]
async fn user_sees_own_history() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("rbac_own");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    // Submit a game request
    client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();

    // Query history without user_id — should return own data
    let res = client
        .get(format!("{}/api/history", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// User cannot see another user's history
// ---------------------------------------------------------------------------

#[tokio::test]
async fn user_cannot_see_other_users_history() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();

    // User A submits a game request
    let user_a = app.unique_name("rbac_a");
    let token_a = app.register_and_login(&user_a, "password123").await;
    client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();

    // User B tries to query with User A's user_id
    let user_b = app.unique_name("rbac_b");
    let token_b = app.register_and_login(&user_b, "password123").await;

    let user_a_id: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE username = $1")
        .bind(&user_a)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/api/history?user_id={}",
            app.base_url, user_a_id.0
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    // User B should see 0 results — the user_id param is ignored for regular users
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        0,
        "regular user must not see another user's history"
    );
}

// ---------------------------------------------------------------------------
// Admin can see any user's history
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_can_query_any_users_history() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();

    // Regular user submits a game request
    let user = app.unique_name("rbac_usr");
    let token_user = app.register_and_login(&user, "password123").await;
    client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token_user}"))
        .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();

    let user_id: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE username = $1")
        .bind(&user)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    // Admin queries that user's history
    let admin = app.unique_name("rbac_adm");
    let admin_token = app.register_admin(&admin, "password123").await;

    let res = client
        .get(format!(
            "{}/api/history?user_id={}",
            app.base_url, user_id.0
        ))
        .header("Authorization", format!("Bearer {admin_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        1,
        "admin should see the user's history"
    );
}

// ---------------------------------------------------------------------------
// Admin can see all history (no user_id filter)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_can_see_all_history() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();

    // Two different users each submit a request
    for i in 0..2 {
        let name = app.unique_name(&format!("rbac_all{i}"));
        let token = app.register_and_login(&name, "password123").await;
        client
            .post(format!("{}/api/game/next", app.base_url))
            .header("Authorization", format!("Bearer {token}"))
            .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
            .send()
            .await
            .unwrap();
    }

    // Admin queries without user_id filter
    let admin = app.unique_name("rbac_alladm");
    let admin_token = app.register_admin(&admin, "password123").await;

    let res = client
        .get(format!("{}/api/history", app.base_url))
        .header("Authorization", format!("Bearer {admin_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    // Should see at least the 2 requests from the two users above
    assert!(
        body["data"].as_array().unwrap().len() >= 2,
        "admin without user_id filter should see all history"
    );
}

// ---------------------------------------------------------------------------
// JWT role claim is included and valid
// ---------------------------------------------------------------------------

#[tokio::test]
async fn login_token_contains_role_claim() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("rbac_claim");
    let token = app.register_and_login(&username, "password123").await;

    // Decode the JWT payload (base64) and verify the role claim
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3);

    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .unwrap();
    let claims: serde_json::Value = serde_json::from_slice(&payload).unwrap();
    assert_eq!(claims["role"], "user");
}

#[tokio::test]
async fn admin_token_contains_admin_role_claim() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("rbac_admclaim");
    let token = app.register_admin(&username, "password123").await;

    let parts: Vec<&str> = token.split('.').collect();
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .unwrap();
    let claims: serde_json::Value = serde_json::from_slice(&payload).unwrap();
    assert_eq!(claims["role"], "admin");
}
