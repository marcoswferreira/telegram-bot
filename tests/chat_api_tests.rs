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

fn test_config(base_url: &str, api_key: Option<&str>) -> Config {
    Config {
        telegram_bot_token: "test-token".to_string(),
        telegram_chat_id: None,
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
async fn get_chat_returns_metadata_and_requires_auth_when_configured() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/getChat"))
        .and(body_json(json!({ "chat_id": "-100123" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": {
                "id": -100123,
                "type": "channel",
                "title": "My Channel",
                "username": "mychannel",
                "description": "Announcements"
            }
        })))
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), Some("secret"));

    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-100123",
        Method::GET,
        &[],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-100123",
        Method::GET,
        &[("x-api-key", "secret")],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], -100123);
    assert_eq!(body["chat_type"], "channel");
    assert_eq!(body["title"], "My Channel");
}

#[tokio::test]
async fn updates_title_and_description_with_validation() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/setChatTitle"))
        .and(body_json(
            json!({ "chat_id": "-1001", "title": "New Title" }),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "result": true })),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/bottest-token/setChatDescription"))
        .and(body_json(
            json!({ "chat_id": "-1001", "description": "Updated desc" }),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "result": true })),
        )
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), None);

    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-1001/title",
        Method::PUT,
        &[],
        Some(json!({ "title": "   " })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-1001/title",
        Method::PUT,
        &[],
        Some(json!({ "title": "New Title" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);

    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-1001/description",
        Method::PUT,
        &[],
        Some(json!({ "description": "Updated desc" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn uploads_and_deletes_chat_photo() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/setChatPhoto"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "result": true })),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/bottest-token/deleteChatPhoto"))
        .and(body_json(json!({ "chat_id": "-1001" })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "result": true })),
        )
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), None);

    // Multipart upload with 'photo' field
    let boundary = "----testboundary";
    let multipart_body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"logo.png\"\r\nContent-Type: image/png\r\n\r\nFAKEPNGDATA\r\n--{boundary}--\r\n"
    );

    let req = Request::builder()
        .method(Method::PUT)
        .uri("/api/v1/chats/-1001/photo")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(multipart_body))
        .unwrap();

    let resp = build_app(&cfg).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Missing 'photo' field -> 400
    let missing_body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"other\"\r\n\r\nhello\r\n--{boundary}--\r\n"
    );
    let req = Request::builder()
        .method(Method::PUT)
        .uri("/api/v1/chats/-1001/photo")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(missing_body))
        .unwrap();
    let resp = build_app(&cfg).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Delete photo
    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-1001/photo",
        Method::DELETE,
        &[],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn pins_unpins_and_creates_invite_links() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/pinChatMessage"))
        .and(body_json(
            json!({ "chat_id": "-1001", "message_id": 55, "disable_notification": true }),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "result": true })),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/bottest-token/unpinChatMessage"))
        .and(body_json(json!({ "chat_id": "-1001" })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "result": true })),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/bottest-token/createChatInviteLink"))
        .and(body_json(
            json!({ "chat_id": "-1001", "name": "VIP", "member_limit": 5 }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "result": {
                "invite_link": "https://t.me/+abc123",
                "name": "VIP",
                "member_limit": 5
            }
        })))
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), None);

    // Invalid message_id -> 400
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-1001/pin",
        Method::POST,
        &[],
        Some(json!({ "message_id": 0 })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Pin message -> 200
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-1001/pin",
        Method::POST,
        &[],
        Some(json!({ "message_id": 55, "disable_notification": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Unpin without body -> 200
    let (status, _) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-1001/pin",
        Method::DELETE,
        &[],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Create invite link -> 200
    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-1001/invite-links",
        Method::POST,
        &[],
        Some(json!({ "name": "VIP", "member_limit": 5 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["invite_link"], "https://t.me/+abc123");
    assert_eq!(body["member_limit"], 5);
}

#[tokio::test]
async fn maps_upstream_chat_errors_to_bad_gateway() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/setChatTitle"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "ok": false,
            "description": "Forbidden: not enough rights"
        })))
        .mount(&server)
        .await;

    let cfg = test_config(&server.uri(), None);
    let (status, body) = send_json(
        build_app(&cfg),
        "/api/v1/chats/-1001/title",
        Method::PUT,
        &[],
        Some(json!({ "title": "Valid Title" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("not enough rights"));
}
