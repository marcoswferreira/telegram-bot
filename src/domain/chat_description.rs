use crate::domain::domain_error::DomainError;

const MAX_DESCRIPTION_LEN: usize = 255;

/// Validated Telegram group/channel description (0..=255 characters).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatDescription(String);

impl ChatDescription {
    pub fn parse(raw: String) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.chars().count() > MAX_DESCRIPTION_LEN {
            return Err(DomainError::Validation(format!(
                "description exceeds {MAX_DESCRIPTION_LEN} character limit"
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
    fn validates_chat_description_length() {
        assert_eq!(ChatDescription::parse("".into()).unwrap().as_str(), "");
        assert_eq!(
            ChatDescription::parse("  Hello  ".into()).unwrap().as_str(),
            "Hello"
        );
        assert!(ChatDescription::parse("a".repeat(256)).is_err());
    }
}
