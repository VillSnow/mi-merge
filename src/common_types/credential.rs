use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Credential {
    pub host: String,
    pub api_key: String,

    #[serde(default)]
    pub disable: bool,
}
