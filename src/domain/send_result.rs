/// Domain result returned after a message is successfully delivered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendResult {
    pub message_id: i64,
    pub chat_id: String,
}
