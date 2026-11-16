use crate::domain::domain_error::DomainError;

/// Supported Telegram message formatting modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    MarkdownV2,
    Html,
    Markdown,
}

impl ParseMode {
    /// Parse a Telegram parse_mode string into `ParseMode`.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "MarkdownV2" => Ok(Self::MarkdownV2),
            "HTML" => Ok(Self::Html),
            "Markdown" => Ok(Self::Markdown),
            _ => Err(DomainError::Validation(
                "parse_mode must be one of: MarkdownV2, HTML, Markdown".to_string(),
            )),
        }
    }

    /// Return the exact Telegram API string representation.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MarkdownV2 => "MarkdownV2",
            Self::Html => "HTML",
            Self::Markdown => "Markdown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_modes_and_rejects_unknown() {
        assert_eq!(ParseMode::parse("MarkdownV2"), Ok(ParseMode::MarkdownV2));
        assert_eq!(ParseMode::parse("HTML"), Ok(ParseMode::Html));
        assert_eq!(ParseMode::parse("Markdown"), Ok(ParseMode::Markdown));
        assert_eq!(ParseMode::MarkdownV2.as_str(), "MarkdownV2");
        assert_eq!(ParseMode::Html.as_str(), "HTML");
        assert_eq!(ParseMode::Markdown.as_str(), "Markdown");
        assert!(matches!(
            ParseMode::parse("BBCode"),
            Err(DomainError::Validation(_))
        ));
    }
}
