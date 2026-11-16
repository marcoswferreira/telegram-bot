/// Domain entity representing a created Telegram invite link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteLink {
    pub invite_link: String,
    pub name: Option<String>,
    pub expire_date: Option<i64>,
    pub member_limit: Option<i64>,
}
