use crate::domain::domain_error::DomainError;

const MAX_TITLE_LEN: usize = 128;

/// Validated Telegram group/channel title (1..=128 characters).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTitle(String);

impl ChatTitle {
    pub fn parse(raw: String) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "title must not be empty".to_string(),
            ));
        }
        if trimmed.chars().count() > MAX_TITLE_LEN {
            return Err(DomainError::Validation(format!(
                "title exceeds {MAX_TITLE_LEN} character limit"
            )));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_chat_title_length() {
        assert_eq!(
            ChatTitle::parse("  My Channel  ".into()).unwrap().as_str(),
            "My Channel"
        );
        assert!(ChatTitle::parse("   ".into()).is_err());
        assert!(ChatTitle::parse("a".repeat(129)).is_err());
    }
}
