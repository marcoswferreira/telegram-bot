use crate::domain::domain_error::DomainError;

/// Validated options for creating a Telegram invite link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteLinkOptions {
    pub name: Option<String>,
    pub expire_date: Option<i64>,
    pub member_limit: Option<i64>,
}

impl InviteLinkOptions {
    pub fn new(
        name: Option<String>,
        expire_date: Option<i64>,
        member_limit: Option<i64>,
    ) -> Result<Self, DomainError> {
        if let Some(limit) = member_limit {
            if !(1..=99999).contains(&limit) {
                return Err(DomainError::Validation(
                    "member_limit must be between 1 and 99999".to_string(),
                ));
            }
        }
        Ok(Self {
            name: name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            expire_date,
            member_limit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_member_limit_range() {
        assert!(InviteLinkOptions::new(None, None, Some(0)).is_err());
        assert!(InviteLinkOptions::new(None, None, Some(100_000)).is_err());
        let opts =
            InviteLinkOptions::new(Some("  vip  ".into()), Some(1_700_000_000), Some(10)).unwrap();
        assert_eq!(opts.name.as_deref(), Some("vip"));
        assert_eq!(opts.member_limit, Some(10));
    }
}
