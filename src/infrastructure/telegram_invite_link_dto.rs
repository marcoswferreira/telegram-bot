use serde::Deserialize;

use crate::domain::invite_link::InviteLink;

/// Telegram Bot API `ChatInviteLink` DTO returned by `createChatInviteLink`.
#[derive(Debug, Deserialize)]
pub struct TelegramInviteLinkDto {
    pub invite_link: String,
    pub name: Option<String>,
    pub expire_date: Option<i64>,
    pub member_limit: Option<i64>,
}

impl From<TelegramInviteLinkDto> for InviteLink {
    fn from(dto: TelegramInviteLinkDto) -> Self {
        Self {
            invite_link: dto.invite_link,
            name: dto.name,
            expire_date: dto.expire_date,
            member_limit: dto.member_limit,
        }
    }
}
