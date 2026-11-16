use std::sync::Arc;

use crate::domain::{
    chat_description::ChatDescription, chat_provisioning_gateway::ChatProvisioningGateway,
    chat_title::ChatTitle, create_channel_request::CreateChannelRequest, created_chat::CreatedChat,
    domain_error::DomainError,
};

/// Use case for creating a new Telegram channel or supergroup via MTProto.
pub struct CreateChannelUseCase {
    gateway: Arc<dyn ChatProvisioningGateway>,
    default_bot_username: Option<String>,
}

impl CreateChannelUseCase {
    pub fn new(
        gateway: Arc<dyn ChatProvisioningGateway>,
        default_bot_username: Option<String>,
    ) -> Self {
        Self {
            gateway,
            default_bot_username,
        }
    }

    pub async fn execute(
        &self,
        raw_title: String,
        raw_description: Option<String>,
        is_supergroup: bool,
        bot_username: Option<String>,
    ) -> Result<CreatedChat, DomainError> {
        let title = ChatTitle::parse(raw_title)?;
        let description = ChatDescription::parse(raw_description.unwrap_or_default())?;
        let resolved_bot = bot_username
            .map(|s| s.trim().trim_start_matches('@').to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                self.default_bot_username
                    .as_deref()
                    .map(|s| s.trim_start_matches('@').to_string())
            });

        let request = CreateChannelRequest {
            title,
            description,
            broadcast: !is_supergroup,
            megagroup: is_supergroup,
            bot_username: resolved_bot,
        };

        self.gateway.create_channel(&request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::chat_gateway::BoxFuture;

    struct FakeProvisioningGateway;

    impl ChatProvisioningGateway for FakeProvisioningGateway {
        fn create_channel<'a>(
            &'a self,
            request: &'a CreateChannelRequest,
        ) -> BoxFuture<'a, Result<CreatedChat, DomainError>> {
            Box::pin(async move {
                Ok(CreatedChat {
                    id: -100777,
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

    #[tokio::test]
    async fn creates_channel_and_supergroup_with_validation() {
        let uc = CreateChannelUseCase::new(
            Arc::new(FakeProvisioningGateway),
            Some("@my_bot".to_string()),
        );

        let ch = uc
            .execute("News".into(), Some("Daily updates".into()), false, None)
            .await
            .unwrap();
        assert_eq!(ch.chat_type, "channel");
        assert_eq!(ch.title, "News");

        let grp = uc
            .execute("Team Chat".into(), None, true, Some("@other_bot".into()))
            .await
            .unwrap();
        assert_eq!(grp.chat_type, "supergroup");

        assert!(uc.execute("   ".into(), None, false, None).await.is_err());
    }
}
