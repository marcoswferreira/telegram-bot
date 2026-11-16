use serde::Serialize;

use crate::domain::invite_link::InviteLink;

/// HTTP response DTO for `POST /api/v1/chats/{chat_id}/invite-links`.
#[derive(Debug, Serialize)]
pub struct InviteLinkHttpResponse {
    pub ok: bool,
    pub invite_link: String,
    pub name: Option<String>,
    pub expire_date: Option<i64>,
    pub member_limit: Option<i64>,
}

impl From<InviteLink> for InviteLinkHttpResponse {
    fn from(link: InviteLink) -> Self {
        Self {
            ok: true,
            invite_link: link.invite_link,
            name: link.name,
            expire_date: link.expire_date,
            member_limit: link.member_limit,
        }
    }
}
