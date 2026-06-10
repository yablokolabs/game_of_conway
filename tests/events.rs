mod common;

use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn events_endpoint_requires_auth() {
    let app = common::TestApp::spawn().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/api/events", app.base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn events_endpoint_returns_event_stream() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("sse");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/api/events", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let ct = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(
        ct.starts_with("text/event-stream"),
        "expected text/event-stream, got {ct}"
    );
}

#[tokio::test]
async fn game_request_broadcasts_event_to_subscribers() {
    let app = common::TestApp::spawn().await;
    let username = app.unique_name("broadcast");
    let token = app.register_and_login(&username, "password123").await;
    let client = reqwest::Client::new();

    // Subscribe to broadcast channel directly
    let mut rx = app.event_tx.subscribe();

    // Submit a game request
    let res = client
        .post(format!("{}/api/game/next", app.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({"cells": [[0,1,0],[0,1,0],[0,1,0]]}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verify event was broadcast
    let event = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
        .await
        .expect("should receive event within timeout")
        .expect("broadcast recv should succeed");

    assert_eq!(event.grid_size, 3);
    assert_eq!(
        event.input_grid,
        vec![vec![0, 1, 0], vec![0, 1, 0], vec![0, 1, 0]]
    );
    assert_eq!(
        event.output_grid,
        vec![vec![0, 0, 0], vec![1, 1, 1], vec![0, 0, 0]]
    );
}
