use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Connection {
    pub host: String,
    pub user: String,

    pub channels: Vec<Channel>,

    #[serde(default)]
    pub disable: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Channel {
    pub channel: ChannelChannel,

    #[serde(default)]
    pub branches: HashSet<String>,

    #[serde(default)]
    pub disable: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "channel")]
pub enum ChannelChannel {
    #[serde(rename = "homeTimeline")]
    HomeTimeline,

    #[serde(rename = "localTimeline")]
    LocalTimeline,

    #[serde(rename = "channel")]
    Channel { channel_id: String },
}
