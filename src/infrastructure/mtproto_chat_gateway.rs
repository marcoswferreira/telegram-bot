use std::{
    io::{self, Write},
    path::Path,
    sync::Arc,
};

use grammers_client::{Client, InvocationError, SenderPool, SignInError};
use grammers_session::{
    storages::SqliteSession,
    types::{PeerAuth, PeerId, PeerRef},
};
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
            .map_err(|e| map_invocation_error("MTProto CreateChannel failed", e))?;

        let (created, channel_ref) = extract_created_chat(updates, request.megagroup)?;

        if let Some(bot_username) = request.bot_username.as_deref() {
            if let Some(ch_ref) = channel_ref {
                self.promote_bot(ch_ref, bot_username).await?;
            }
        }

        Ok(created)
    }

    async fn promote_bot(
        &self,
        channel_ref: PeerRef,
        bot_username: &str,
    ) -> Result<(), DomainError> {
        let bot_peer = self
            .client
            .resolve_username(bot_username)
            .await
            .map_err(|e| map_invocation_error("Failed to resolve bot_username", e))?
            .ok_or_else(|| {
                DomainError::Validation(format!("Bot username '@{bot_username}' not found"))
            })?;

        let bot_ref = bot_peer
            .to_ref()
            .await
            .map_err(|e| DomainError::Upstream(format!("Failed to resolve bot peer ref: {e}")))?
            .ok_or_else(|| DomainError::Upstream("Bot peer missing access_hash".to_string()))?;

        self.client
            .set_admin_rights(channel_ref, bot_ref)
            .change_info(true)
            .post_messages(true)
            .edit_messages(true)
            .delete_messages(true)
            .invite_users(true)
            .pin_messages(true)
            .await
            .map_err(|e| map_invocation_error("Failed to promote bot as admin", e))?;

        Ok(())
    }
}

fn map_invocation_error(context: &str, err: InvocationError) -> DomainError {
    if let InvocationError::Rpc(ref rpc) = err {
        if rpc.name == "FLOOD_WAIT" {
            return DomainError::RateLimited {
                message: format!("{context}: FLOOD_WAIT"),
                retry_after: rpc.value.map(u64::from),
            };
        }
    }
    DomainError::Upstream(format!("{context}: {err}"))
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
) -> Result<(CreatedChat, Option<PeerRef>), DomainError> {
    let chats = match updates {
        tl::enums::Updates::Updates(u) => u.chats,
        tl::enums::Updates::Combined(u) => u.chats,
        _ => vec![],
    };

    for chat in chats {
        if let tl::enums::Chat::Channel(ch) = chat {
            let bot_api_id = format!("-100{}", ch.id).parse::<i64>().unwrap_or(-ch.id);
            let peer_ref = ch.access_hash.and_then(|hash| {
                PeerId::channel(ch.id).map(|id| PeerRef {
                    id,
                    auth: PeerAuth::from_hash(hash),
                })
            });
            return Ok((
                CreatedChat {
                    id: bot_api_id,
                    title: ch.title,
                    chat_type: if is_supergroup {
                        "supergroup".to_string()
                    } else {
                        "channel".to_string()
                    },
                },
                peer_ref,
            ));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_channel(id: i64, title: &str) -> tl::enums::Chat {
        tl::enums::Chat::Channel(tl::types::Channel {
            creator: true,
            left: false,
            broadcast: true,
            verified: false,
            megagroup: false,
            restricted: false,
            signatures: false,
            min: false,
            scam: false,
            has_link: false,
            has_geo: false,
            slowmode_enabled: false,
            call_active: false,
            call_not_empty: false,
            fake: false,
            gigagroup: false,
            noforwards: false,
            join_to_send: false,
            join_request: false,
            forum: false,
            stories_hidden: false,
            stories_hidden_min: false,
            stories_unavailable: false,
            signature_profiles: false,
            autotranslation: false,
            broadcast_messages_allowed: false,
            monoforum: false,
            forum_tabs: false,
            in_news: false,
            forum_sidebar: false,
            id,
            access_hash: None,
            title: title.to_string(),
            username: None,
            photo: tl::enums::ChatPhoto::Empty,
            date: 0,
            restriction_reason: None,
            admin_rights: None,
            banned_rights: None,
            default_banned_rights: None,
            participants_count: None,
            usernames: None,
            stories_max_id: None,
            color: None,
            profile_color: None,
            emoji_status: None,
            level: None,
            subscription_until_date: None,
            bot_verification_icon: None,
            send_paid_messages_stars: None,
            linked_monoforum_id: None,
            linked_community_id: None,
            pinned_msg_id: None,
        })
    }

    #[test]
    fn extracts_created_channel_and_group_from_updates() {
        let updates = tl::enums::Updates::Updates(tl::types::Updates {
            updates: vec![],
            users: vec![],
            chats: vec![sample_channel(123456, "My Channel")],
            date: 0,
            seq: 0,
        });
        let (created, _) = extract_created_chat(updates, false).unwrap();
        assert_eq!(created.id, -100123456);
        assert_eq!(created.title, "My Channel");
        assert_eq!(created.chat_type, "channel");

        let combined = tl::enums::Updates::Combined(tl::types::UpdatesCombined {
            updates: vec![],
            users: vec![],
            chats: vec![sample_channel(999, "My Group")],
            date: 0,
            seq_start: 0,
            seq: 0,
        });
        let (grp, _) = extract_created_chat(combined, true).unwrap();
        assert_eq!(grp.id, -100999);
        assert_eq!(grp.chat_type, "supergroup");

        let empty = tl::enums::Updates::TooLong;
        assert!(matches!(
            extract_created_chat(empty, false),
            Err(DomainError::Upstream(_))
        ));
    }
}
