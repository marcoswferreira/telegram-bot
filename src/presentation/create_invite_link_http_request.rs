use serde::Deserialize;

/// Optional HTTP request DTO for `POST /api/v1/chats/{chat_id}/invite-links`.
#[derive(Debug, Default, Deserialize)]
pub struct CreateInviteLinkHttpRequest {
    pub name: Option<String>,
    pub expire_date: Option<i64>,
    pub member_limit: Option<i64>,
}
