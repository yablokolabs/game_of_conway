mod common;

use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn register_creates_user_and_returns_id() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();
    let username = app.unique_name("reg");

    let res = client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&json!({"username": username, "password": "password123"}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["username"], username);
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn register_rejects_duplicate_username() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();
    let username = app.unique_name("dup");
    let body = json!({"username": username, "password": "password123"});

    let first = client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);

    let second = client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn register_rejects_short_password() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();
    let username = app.unique_name("short");

    let res = client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&json!({"username": username, "password": "short"}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn login_returns_jwt_token() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();
    let username = app.unique_name("login");

    client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&json!({"username": username, "password": "password123"}))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({"username": username, "password": "password123"}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    let token = body["token"].as_str().unwrap();
    assert!(!token.is_empty());
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();
    let username = app.unique_name("wrongpw");

    client
        .post(format!("{}/api/auth/register", app.base_url))
        .json(&json!({"username": username, "password": "password123"}))
        .send()
        .await
        .unwrap();

    let res = client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({"username": username, "password": "wrong"}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_rejects_nonexistent_user() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/auth/login", app.base_url))
        .json(&json!({"username": "ghost_user", "password": "password123"}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
