use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use telegram_bot::{build_app, Config};
use tower::ServiceExt;
use wiremock::{
    matchers::{body_json, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn test_config(base_url: &str, default_chat_id: Option<&str>) -> Config {
    Config {
        telegram_bot_token: "test-token".to_string(),
        telegram_chat_id: default_chat_id.map(str::to_string),
        api_key: None,
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        telegram_base_url: base_url.to_string(),
        telegram_api_id: None,
        telegram_api_hash: None,
        telegram_session_path: "./data/test.session".to_string(),
        telegram_bot_username: None,
    }
}

#[tokio::test]
async fn edits_and_deletes_messages() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/editMessageText"))
        .and(body_json(json!({
            "chat_id": "-1001",
            "message_id": 42,
            "text": "Edited text",
            "parse_mode": "HTML",
            "disable_web_page_preview": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "result": {} })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/bottest-token/deleteMessage"))
        .and(body_json(json!({ "chat_id": "-1001", "message_id": 42 })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "result": true })),
        )
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), None);

    let req = Request::builder()
        .method(Method::PUT)
        .uri("/api/v1/chats/-1001/messages/42")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "text": "Edited text",
                "parse_mode": "HTML",
                "disable_web_page_preview": true
            })
            .to_string(),
        ))
        .unwrap();
    let resp = build_app(&cfg).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/chats/-1001/messages/42")
        .body(Body::empty())
        .unwrap();
    let resp = build_app(&cfg).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn sends_photo_and_document_multipart() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendPhoto"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": { "message_id": 88 }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/bottest-token/sendDocument"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": { "message_id": 89 }
        })))
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), Some("-1001"));
    let boundary = "----mediaboundary";

    let photo_body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"caption\"\r\n\r\nChart\r\n\
         --{boundary}\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"chart.png\"\r\nContent-Type: image/png\r\n\r\nPNG\r\n--{boundary}--\r\n"
    );
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/messages/photo")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(photo_body))
        .unwrap();
    let resp = build_app(&cfg).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["message_id"], 88);

    let doc_body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"document\"; filename=\"report.pdf\"\r\nContent-Type: application/pdf\r\n\r\nPDF\r\n--{boundary}--\r\n"
    );
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/messages/document")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(doc_body))
        .unwrap();
    let resp = build_app(&cfg).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn maps_telegram_429_to_too_many_requests_with_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "ok": false,
            "description": "Too Many Requests: retry after 15",
            "parameters": { "retry_after": 15 }
        })))
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), Some("-1001"));
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/messages")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "text": "hello" }).to_string()))
        .unwrap();
    let resp = build_app(&cfg).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        resp.headers()
            .get(header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok()),
        Some("15")
    );
}
