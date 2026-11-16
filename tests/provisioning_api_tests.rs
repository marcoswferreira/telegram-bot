use std::sync::Arc;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use telegram_bot::{
    build_app, build_app_with_provisioning,
    domain::{
        chat_gateway::BoxFuture, chat_provisioning_gateway::ChatProvisioningGateway,
        create_channel_request::CreateChannelRequest, created_chat::CreatedChat,
        domain_error::DomainError,
    },
    Config,
};
use tower::ServiceExt;

fn test_config(api_key: Option<&str>) -> Config {
    Config {
        telegram_bot_token: "test-token".to_string(),
        telegram_chat_id: None,
        api_key: api_key.map(str::to_string),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        telegram_base_url: "http://127.0.0.1:1".to_string(),
        telegram_api_id: None,
        telegram_api_hash: None,
        telegram_session_path: "./data/test.session".to_string(),
    }
}

struct MockProvisioningGateway;

impl ChatProvisioningGateway for MockProvisioningGateway {
    fn create_channel<'a>(
        &'a self,
        request: &'a CreateChannelRequest,
    ) -> BoxFuture<'a, Result<CreatedChat, DomainError>> {
        Box::pin(async move {
            Ok(CreatedChat {
                id: -100555666,
                title: request.title.as_str().to_string(),
                chat_type: if request.megagroup {
                    "supergroup".to_string()
                } else {
                    "channel".to_string()
                },
            })
        })
    }
}

async fn post_json(
    app: axum::Router,
    uri: &str,
    headers: &[(&str, &str)],
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("content-type", "application/json");
    for (k, v) in headers {
        builder = builder.header(*k, *v);
    }
    let req = builder.body(Body::from(body.to_string())).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn returns_validation_error_when_mtproto_is_not_configured() {
    let cfg = test_config(None);
    let (status, body) = post_json(
        build_app(&cfg),
        "/api/v1/chats/channels",
        &[],
        json!({ "title": "My Channel" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("MTProto is not configured"));
}

#[tokio::test]
async fn creates_channel_and_group_when_mtproto_is_enabled() {
    let cfg = test_config(Some("secret"));
    let gw = Arc::new(MockProvisioningGateway);

    // Requires auth
    let (status, _) = post_json(
        build_app_with_provisioning(&cfg, gw.clone()),
        "/api/v1/chats/channels",
        &[],
        json!({ "title": "News" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Create channel
    let (status, body) = post_json(
        build_app_with_provisioning(&cfg, gw.clone()),
        "/api/v1/chats/channels",
        &[("x-api-key", "secret")],
        json!({ "title": "News", "description": "Updates" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["ok"], true);
    assert_eq!(body["id"], -100555666);
    assert_eq!(body["chat_type"], "channel");

    // Create supergroup
    let (status, body) = post_json(
        build_app_with_provisioning(&cfg, gw),
        "/api/v1/chats/groups",
        &[("x-api-key", "secret")],
        json!({ "title": "Dev Team" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["chat_type"], "supergroup");
}
