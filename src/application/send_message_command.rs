/// Application input command for sending a message.
#[derive(Debug, Clone)]
pub struct SendMessageCommand {
    pub chat_id: Option<String>,
    pub text: String,
    pub parse_mode: Option<String>,
    pub disable_web_page_preview: Option<bool>,
    pub disable_notification: Option<bool>,
}
