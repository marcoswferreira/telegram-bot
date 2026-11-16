use std::{
    io::{self, Write},
    path::Path,
    sync::Arc,
};

use grammers_client::{Client, SenderPool, SignInError};
use grammers_session::storages::SqliteSession;
use grammers_tl_types as tl;

use crate::domain::{
    chat_gateway::BoxFuture, chat_provisioning_gateway::ChatProvisioningGateway,
    create_channel_request::CreateChannelRequest, created_chat::CreatedChat,
    domain_error::DomainError,
};

/// MTProto adapter implementing `ChatProvisioningGateway` via `grammers-client`.
#[derive(Clone)]
pub struct MtprotoChatGateway {
    client: Client,
}

impl MtprotoChatGateway {
    /// Connect to Telegram MTProto using a persistent SQLite session file.
    pub async fn connect(api_id: i32, session_path: &str) -> Result<Self, String> {
        if let Some(parent) = Path::new(session_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create session dir: {e}"))?;
            }
        }

        let session = Arc::new(
            SqliteSession::open(session_path)
                .await
                .map_err(|e| format!("Failed to open MTProto session '{session_path}': {e}"))?,
        );
        let pool = SenderPool::new(Arc::clone(&session), api_id);
        let client = Client::new(pool.handle);

        tokio::spawn(async move {
            pool.runner.run().await;
        });

        Ok(Self { client })
    }

    /// Run an interactive CLI login flow to authorize and persist the MTProto session.
    pub async fn login_interactive(
        api_id: i32,
        api_hash: &str,
        session_path: &str,
    ) -> Result<(), String> {
        let gateway = Self::connect(api_id, session_path).await?;
        if gateway
            .client
            .is_authorized()
            .await
            .map_err(|e| format!("Authorization check failed: {e}"))?
        {
            println!("MTProto session is already authorized at {session_path}");
            return Ok(());
        }

        let phone = prompt("Enter phone number (with country code, e.g. +5511999999999): ")?;
        let token = gateway
            .client
            .request_login_code(&phone, api_hash)
            .await
            .map_err(|e| format!("Failed to request login code: {e}"))?;

        let code = prompt("Enter the login code you received: ")?;
        match gateway.client.sign_in(&token, &code).await {
            Ok(_) => {}
            Err(SignInError::PasswordRequired(password_token)) => {
                let hint = password_token.hint().unwrap_or("none");
                let password = prompt(&format!("Enter 2FA password (hint: {hint}): "))?;
                gateway
                    .client
                    .check_password(password_token, password)
                    .await
                    .map_err(|e| format!("2FA check failed: {e}"))?;
            }
            Err(e) => return Err(format!("Sign-in failed: {e}")),
        }

        println!("MTProto session authorized and saved to {session_path}");
        Ok(())
    }

    async fn create_channel_internal(
        &self,
        request: &CreateChannelRequest,
    ) -> Result<CreatedChat, DomainError> {
        let authorized = self
            .client
            .is_authorized()
            .await
            .map_err(|e| DomainError::Upstream(format!("MTProto auth check failed: {e}")))?;

        if !authorized {
            return Err(DomainError::Unauthorized(
                "MTProto session is not authorized; run with --mtproto-login first".to_string(),
            ));
        }

        let updates = self
            .client
            .invoke(&tl::functions::channels::CreateChannel {
                broadcast: request.broadcast,
                megagroup: request.megagroup,
                for_import: false,
                forum: false,
                title: request.title.as_str().to_string(),
                about: request.description.as_str().to_string(),
                geo_point: None,
                address: None,
                ttl_period: None,
            })
            .await
            .map_err(|e| DomainError::Upstream(format!("MTProto CreateChannel failed: {e}")))?;

        extract_created_chat(updates, request.megagroup)
    }
}

impl ChatProvisioningGateway for MtprotoChatGateway {
    fn create_channel<'a>(
        &'a self,
        request: &'a CreateChannelRequest,
    ) -> BoxFuture<'a, Result<CreatedChat, DomainError>> {
        Box::pin(self.create_channel_internal(request))
    }
}

fn extract_created_chat(
    updates: tl::enums::Updates,
    is_supergroup: bool,
) -> Result<CreatedChat, DomainError> {
    let chats = match updates {
        tl::enums::Updates::Updates(u) => u.chats,
        tl::enums::Updates::Combined(u) => u.chats,
        _ => vec![],
    };

    for chat in chats {
        if let tl::enums::Chat::Channel(ch) = chat {
            let bot_api_id = format!("-100{}", ch.id).parse::<i64>().unwrap_or(-ch.id);
            return Ok(CreatedChat {
                id: bot_api_id,
                title: ch.title,
                chat_type: if is_supergroup {
                    "supergroup".to_string()
                } else {
                    "channel".to_string()
                },
            });
        }
    }

    Err(DomainError::Upstream(
        "MTProto CreateChannel response did not contain created channel".to_string(),
    ))
}

fn prompt(msg: &str) -> Result<String, String> {
    print!("{msg}");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .map_err(|e| format!("Failed to read stdin: {e}"))?;
    Ok(buf.trim().to_string())
}
