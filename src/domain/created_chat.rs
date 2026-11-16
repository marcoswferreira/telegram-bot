/// Domain entity representing a newly created Telegram group or channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedChat {
    pub id: i64,
    pub title: String,
    pub chat_type: String,
}
