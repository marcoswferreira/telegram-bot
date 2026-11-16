use std::net::SocketAddr;

/// Runtime configuration loaded from environment variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub telegram_bot_token: String,
    pub telegram_chat_id: Option<String>,
    pub api_key: Option<String>,
    pub listen_addr: SocketAddr,
    pub telegram_base_url: String,
    pub telegram_api_id: Option<i32>,
    pub telegram_api_hash: Option<String>,
    pub telegram_session_path: String,
}

impl Config {
    /// Load configuration from `.env` (if present) and process environment variables.
    pub fn from_env() -> Result<Self, String> {
        let _ = dotenvy::dotenv();
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Parse configuration from an arbitrary key-lookup function (test-friendly).
    pub fn from_lookup<F>(lookup: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let telegram_bot_token = non_empty(lookup("TELEGRAM_BOT_TOKEN"))
            .ok_or_else(|| "TELEGRAM_BOT_TOKEN must be set and non-empty".to_string())?;

        let telegram_chat_id = non_empty(lookup("TELEGRAM_CHAT_ID"));
        let api_key = non_empty(lookup("API_KEY"));

        let listen_raw =
            non_empty(lookup("LISTEN_ADDR")).unwrap_or_else(|| "0.0.0.0:8080".to_string());
        let listen_addr = listen_raw
            .parse::<SocketAddr>()
            .map_err(|e| format!("Invalid LISTEN_ADDR '{listen_raw}': {e}"))?;

        let telegram_base_url = non_empty(lookup("TELEGRAM_BASE_URL"))
            .unwrap_or_else(|| "https://api.telegram.org".to_string());

        let telegram_api_id = match non_empty(lookup("TELEGRAM_API_ID")) {
            Some(raw) => Some(
                raw.parse::<i32>()
                    .map_err(|e| format!("Invalid TELEGRAM_API_ID '{raw}': {e}"))?,
            ),
            None => None,
        };
        let telegram_api_hash = non_empty(lookup("TELEGRAM_API_HASH"));
        let telegram_session_path = non_empty(lookup("TELEGRAM_SESSION_PATH"))
            .unwrap_or_else(|| "./data/telegram.session".to_string());

        Ok(Self {
            telegram_bot_token,
            telegram_chat_id,
            api_key,
            listen_addr,
            telegram_base_url,
            telegram_api_id,
            telegram_api_hash,
            telegram_session_path,
        })
    }

    /// Return MTProto credentials `(api_id, api_hash)` if both are configured.
    pub fn mtproto_credentials(&self) -> Option<(i32, &str)> {
        match (self.telegram_api_id, self.telegram_api_hash.as_deref()) {
            (Some(id), Some(hash)) => Some((id, hash)),
            _ => None,
        }
    }
}

fn non_empty(val: Option<String>) -> Option<String> {
    val.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn lookup_from(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |k: &str| map.get(k).cloned()
    }

    #[test]
    fn loads_defaults_when_only_token_is_set() {
        let cfg = Config::from_lookup(lookup_from(&[("TELEGRAM_BOT_TOKEN", "tok:123")])).unwrap();
        assert_eq!(cfg.telegram_bot_token, "tok:123");
        assert_eq!(cfg.telegram_chat_id, None);
        assert_eq!(cfg.api_key, None);
        assert_eq!(cfg.listen_addr, "0.0.0.0:8080".parse().unwrap());
        assert_eq!(cfg.telegram_base_url, "https://api.telegram.org");
        assert_eq!(cfg.mtproto_credentials(), None);
        assert_eq!(cfg.telegram_session_path, "./data/telegram.session");
    }

    #[test]
    fn loads_all_custom_fields_and_mtproto_credentials() {
        let cfg = Config::from_lookup(lookup_from(&[
            ("TELEGRAM_BOT_TOKEN", "  tok:456  "),
            ("TELEGRAM_CHAT_ID", "-100999"),
            ("API_KEY", "   "),
            ("LISTEN_ADDR", "127.0.0.1:9090"),
            ("TELEGRAM_BASE_URL", "http://localhost:9999"),
            ("TELEGRAM_API_ID", "12345"),
            ("TELEGRAM_API_HASH", "abcdef"),
            ("TELEGRAM_SESSION_PATH", "/data/tg.session"),
        ]))
        .unwrap();
        assert_eq!(cfg.telegram_bot_token, "tok:456");
        assert_eq!(cfg.telegram_chat_id.as_deref(), Some("-100999"));
        assert_eq!(cfg.api_key, None);
        assert_eq!(cfg.listen_addr, "127.0.0.1:9090".parse().unwrap());
        assert_eq!(cfg.telegram_base_url, "http://localhost:9999");
        assert_eq!(cfg.mtproto_credentials(), Some((12345, "abcdef")));
        assert_eq!(cfg.telegram_session_path, "/data/tg.session");
    }

    #[test]
    fn rejects_missing_or_empty_token_and_invalid_numeric_fields() {
        assert!(Config::from_lookup(lookup_from(&[])).is_err());
        assert!(Config::from_lookup(lookup_from(&[("TELEGRAM_BOT_TOKEN", "  ")])).is_err());

        let err = Config::from_lookup(lookup_from(&[
            ("TELEGRAM_BOT_TOKEN", "tok:123"),
            ("LISTEN_ADDR", "not-an-addr"),
        ]))
        .unwrap_err();
        assert!(err.contains("Invalid LISTEN_ADDR"));

        let err = Config::from_lookup(lookup_from(&[
            ("TELEGRAM_BOT_TOKEN", "tok:123"),
            ("TELEGRAM_API_ID", "abc"),
        ]))
        .unwrap_err();
        assert!(err.contains("Invalid TELEGRAM_API_ID"));
    }
}
