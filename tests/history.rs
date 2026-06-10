mod common;

use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn history_returns_past_requests() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("hist");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    // Submit two game requests
    for _ in 0..2 {
        client
            .post(format!("{}/api/game/next", app.base_url))
            .header("Authorization", format!("Bearer {token}"))
            .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
            .send()
            .await
            .unwrap();
    }

    // Query the user's own history by looking up user_id first
    let user: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    let res = client
        .get(format!("{}/api/history?user_id={}", app.base_url, user.0))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["page"], 1);
}

#[tokio::test]
async fn history_filters_by_grid_size() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("histsz");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    // Submit a 3x3 grid
    client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();

    // Submit a 4x4 grid
    client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [
            [0,0,0,0],
            [0,1,1,0],
            [0,1,1,0],
            [0,0,0,0]
        ]}))
        .send()
        .await
        .unwrap();

    let user: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    // Filter for grid_size=4 only
    let res = client
        .get(format!(
            "{}/api/history?user_id={}&grid_size=4",
            app.base_url, user.0
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["grid_size"], 4);
}

#[tokio::test]
async fn history_requires_auth() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/api/history", app.base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn history_paginates_results() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("histpage");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    // Submit 3 requests
    for _ in 0..3 {
        client
            .post(format!("{}/api/game/next", app.base_url))
            .header("Authorization", format!("Bearer {token}"))
            .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
            .send()
            .await
            .unwrap();
    }

    let user: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE username = $1")
        .bind(&username)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    let res = client
        .get(format!(
            "{}/api/history?user_id={}&per_page=2&page=1",
            app.base_url, user.0
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["per_page"], 2);
}
