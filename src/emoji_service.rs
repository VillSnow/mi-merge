use std::{collections::HashMap, error::Error};

use serde_json::json;
use tracing::info;

use crate::{common_types::Host, mi_models::EmojiSimple};

#[derive(Debug, Clone, Default)]
pub struct EmojiService {
    cache: HashMap<(Host, String), EmojiSimple>,
}

#[derive(Debug)]
pub enum EmojiServiceError {
    HttpRequestError,
    InvalidFormatResponse,
}

impl std::fmt::Display for EmojiServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EmojiServiceError::HttpRequestError => {
                write!(f, "http request note")
            }
            EmojiServiceError::InvalidFormatResponse => {
                write!(f, "invalid format response")
            }
        }
    }
}

impl Error for EmojiServiceError {}

impl EmojiService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn fetch(
        &mut self,
        host: &Host,
        name: &str,
    ) -> Result<EmojiSimple, EmojiServiceError> {
        use std::collections::hash_map::Entry;

        let key = (host.clone(), name.to_owned());
        let entry = self.cache.entry(key);
        let entry = match entry {
            Entry::Occupied(cached) => return Ok(cached.get().clone()),
            Entry::Vacant(entry) => entry,
        };

        info!("fetching emoji info of :{}@{}:", name, host);

        let res = reqwest::Client::new()
            .post(format!("https://{host}/api/emoji"))
            .json(&json!({"name": name}))
            .send()
            .await
            .map_err(|_e| EmojiServiceError::HttpRequestError)?
            .error_for_status()
            .map_err(|_e| EmojiServiceError::HttpRequestError)?;

        let text = res
            .text()
            .await
            .map_err(|_e| EmojiServiceError::HttpRequestError)?;
        dbg!(&text);
        let emoji: EmojiSimple =
            serde_json::from_str(&text).map_err(|_e| EmojiServiceError::InvalidFormatResponse)?;

        entry.insert(emoji.clone());
        Ok(emoji)
    }
}
