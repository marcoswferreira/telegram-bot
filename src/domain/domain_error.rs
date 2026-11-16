/// Core domain errors independent of transport or delivery mechanisms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    Validation(String),
    Unauthorized(String),
    Upstream(String),
}
