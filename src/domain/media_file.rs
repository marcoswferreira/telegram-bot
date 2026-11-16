use crate::domain::domain_error::DomainError;

/// Validated binary media payload for sending photos or documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaFile {
    pub filename: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

impl MediaFile {
    pub fn new(
        filename: Option<String>,
        content_type: Option<String>,
        bytes: Vec<u8>,
        default_filename: &str,
    ) -> Result<Self, DomainError> {
        if bytes.is_empty() {
            return Err(DomainError::Validation(
                "file must not be empty".to_string(),
            ));
        }

        let resolved_name = filename
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default_filename.to_string());

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
    fn validates_media_file() {
        assert!(MediaFile::new(None, None, vec![], "file.bin").is_err());
        let f = MediaFile::new(None, None, vec![1, 2], "photo.jpg").unwrap();
        assert_eq!(f.filename, "photo.jpg");
        assert_eq!(f.content_type, "application/octet-stream");
    }
}
