use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use telegram_bot::{build_app, Config};
use tower::ServiceExt;
use wiremock::{
    matchers::{body_json, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn test_config(base_url: &str, default_chat_id: Option<&str>, api_key: Option<&str>) -> Config {
    Config {
        telegram_bot_token: "test-token".to_string(),
        telegram_chat_id: default_chat_id.map(str::to_string),
        api_key: api_key.map(str::to_string),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        telegram_base_url: base_url.to_string(),
    }
}

async fn send_json(
    app: axum::Router,
    uri: &str,
    method: Method,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    for (k, v) in headers {
        builder = builder.header(*k, *v);
    }

    let req = match body {
        Some(b) => builder
            .header("content-type", "application/json")
            .body(Body::from(b.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };

    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn health_returns_ok_without_auth() {
    let cfg = test_config("http://127.0.0.1:1", None, Some("secret"));
    let app = build_app(&cfg);

    let (status, body) = send_json(app, "/health", Method::GET, &[], None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn auth_middleware_enforces_api_key_when_configured() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": { "message_id": 101 }
        })))
        .expect(3)
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), Some("-1001"), Some("super-secret"));

    // Missing header -> 401
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": "hi" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Blank header -> 401
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[("x-api-key", "   ")],
        Some(json!({ "text": "hi" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Wrong key -> 401
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[("x-api-key", "wrong")],
        Some(json!({ "text": "hi" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Valid X-API-Key -> 200
    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[("x-api-key", "super-secret")],
        Some(json!({ "text": "hi" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["message_id"], 101);

    // Valid Authorization: Bearer -> 200
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[("authorization", "Bearer super-secret")],
        Some(json!({ "text": "hi" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Valid lowercase bearer -> 200
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[("authorization", "bearer super-secret")],
        Some(json!({ "text": "hi" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn sends_with_default_chat_id_and_overrides() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .and(body_json(json!({
            "chat_id": "-100123",
            "text": "default target"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": { "message_id": 1 }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .and(body_json(json!({
            "chat_id": "@mychannel",
            "text": "string override",
            "parse_mode": "HTML",
            "disable_web_page_preview": true,
            "disable_notification": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": { "message_id": 2 }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .and(body_json(json!({
            "chat_id": "-100999888",
            "text": "numeric override"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": { "message_id": 3 }
        })))
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), Some("-100123"), None);

    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": "default target" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["chat_id"], "-100123");
    assert_eq!(body["message_id"], 1);

    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({
            "text": "string override",
            "chat_id": "@mychannel",
            "parse_mode": "HTML",
            "disable_web_page_preview": true,
            "disable_notification": true
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["chat_id"], "@mychannel");
    assert_eq!(body["message_id"], 2);

    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({
            "text": "numeric override",
            "chat_id": -100999888_i64
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["chat_id"], "-100999888");
    assert_eq!(body["message_id"], 3);
}

#[tokio::test]
async fn validates_bad_requests() {
    let cfg = test_config("http://127.0.0.1:1", None, None);

    // Missing chat_id when no default is configured
    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("chat_id"));

    // Empty text
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": "   ", "chat_id": "-1001" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Oversized text (> 4096 chars)
    let long_text = "a".repeat(4097);
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": long_text, "chat_id": "-1001" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Invalid parse_mode
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": "hi", "chat_id": "-1001", "parse_mode": "BBCode" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn maps_upstream_telegram_failures_to_bad_gateway() {
    let server = MockServer::start().await;

    // Telegram 400 error with description
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "ok": false,
            "description": "Bad Request: chat not found"
        })))
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), Some("-1001"), None);
    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert!(body["error"].as_str().unwrap().contains("chat not found"));

    // Invalid JSON from Telegram
    server.reset().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&server)
        .await;

    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);

    // ok: true but missing result
    server.reset().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true })))
        .mount(&server)
        .await;

    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);

    // Unreachable upstream
    let unreachable_cfg = test_config("http://127.0.0.1:1", Some("-1001"), None);
    let (status, _) = send_json(
        build_app(&unreachable_cfg),
        "/api/v1/messages",
        Method::POST,
        &[],
        Some(json!({ "text": "hello" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
}
