/// Domain entity representing Telegram group or channel metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatInfo {
    pub id: i64,
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
}
