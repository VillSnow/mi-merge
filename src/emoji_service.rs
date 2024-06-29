use std::{collections::HashMap, error::Error};

use serde_json::json;

use crate::{common_types::Host, mi_models::Emoji};

#[derive(Debug, Clone, Default)]
pub struct EmojiService {
    cache: HashMap<(Host, String), Emoji>,
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

    pub fn get(&self, host: &Host, name: &str) -> Option<&Emoji> {
        self.cache.get(&(host.clone(), name.to_owned()))
    }

    pub fn insert(&mut self, host: Host, name: String, emoji: Emoji) {
        self.cache.insert((host.clone(), name.to_owned()), emoji);
    }

    pub async fn fetch(host: &Host, name: &str) -> Result<Emoji, EmojiServiceError> {
        let res = reqwest::Client::new()
            .post(format!("https://{host}/emoji"))
            .body(json!({"name": name}).to_string())
            .send()
            .await
            .map_err(|_e| EmojiServiceError::HttpRequestError)?;
        let text = res
            .text()
            .await
            .map_err(|_e| EmojiServiceError::HttpRequestError)?;
        let emoji: Emoji =
            serde_json::from_str(&text).map_err(|_e| EmojiServiceError::InvalidFormatResponse)?;
        Ok(emoji)
    }
}
