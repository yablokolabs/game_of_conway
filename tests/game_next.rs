mod common;

use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn next_state_returns_blinker_oscillation() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("blinker");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["cells"], json!([[0, 0, 0], [1, 1, 1], [0, 0, 0]]));
}

#[tokio::test]
async fn next_state_accepts_boolean_grid() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("bool");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [
            [false, true, false],
            [false, true, false],
            [false, true, false]
        ]}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["cells"], json!([[0, 0, 0], [1, 1, 1], [0, 0, 0]]));
}

#[tokio::test]
async fn next_state_requires_auth() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/game/next", app.base_url))
        .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn next_state_rejects_grid_too_small() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("small");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [[0,1],[1,0]]}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn next_state_rejects_non_square_grid() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("nonsq");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [[0,1,0,0],[0,1,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn next_state_rejects_invalid_cell_values() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("badval");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [[0,1,0],[0,5,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn next_state_persists_request_to_database() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("persist");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM grid_requests gr JOIN users u ON gr.user_id = u.id WHERE u.username = $1")
        .bind(&username)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(count.0, 1, "request should be persisted in the database");
}
