use crate::domain::domain_error::DomainError;

/// Validated image payload for updating a group or channel photo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPhoto {
    pub filename: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

impl ChatPhoto {
    pub fn new(
        filename: Option<String>,
        content_type: Option<String>,
        bytes: Vec<u8>,
    ) -> Result<Self, DomainError> {
        if bytes.is_empty() {
            return Err(DomainError::Validation(
                "photo file must not be empty".to_string(),
            ));
        }

        let resolved_name = filename
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "photo.jpg".to_string());

        let resolved_type = content_type
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        Ok(Self {
            filename: resolved_name,
            content_type: resolved_type,
            bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_non_empty_photo_and_defaults() {
        assert!(ChatPhoto::new(None, None, vec![]).is_err());

        let photo = ChatPhoto::new(None, None, vec![1, 2, 3]).unwrap();
        assert_eq!(photo.filename, "photo.jpg");
        assert_eq!(photo.content_type, "application/octet-stream");
        assert_eq!(photo.bytes, vec![1, 2, 3]);
    }
}
