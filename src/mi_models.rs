use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cw: Option<String>,

    pub user: User,

    #[serde(rename = "userId")]
    pub user_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<Box<Note>>,

    #[serde(rename = "replyId")]
    pub reply_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub renote: Option<Box<Note>>,

    #[serde(rename = "renoteId")]
    pub renote_id: Option<String>,

    pub files: Vec<DriveFile>,

    #[serde(rename = "fileIds")]
    pub file_ids: Vec<String>,

    pub visibility: Visibility,

    #[serde(rename = "visibleUserIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible_user_ids: Option<Vec<String>>,

    #[serde(rename = "localOnly")]
    pub local_only: Option<bool>,

    #[serde(rename = "myReaction")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub my_reaction: Option<String>,

    pub reactions: HashMap<String, i64>,

    #[serde(rename = "renoteCount")]
    pub renote_count: i64,

    #[serde(rename = "repliesCount")]
    pub replies_count: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll: Option<Poll>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub emojis: Option<Vec<Emoji>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(rename = "isHidden")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hidden: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,

    pub username: String,

    pub host: Option<String>,

    pub name: Option<String>,

    #[serde(rename = "onlineStatus")]
    pub online_status: OnlineStatus,

    #[serde(rename = "avatarUrl")]
    pub avatar_url: String,

    #[serde(rename = "avatarBlurhash")]
    pub avatar_blurhash: Option<String>,

    // emojis: Vec<Emoji>,
    pub instance: Option<UserInstance>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum OnlineStatus {
    #[serde(rename = "online")]
    Online,

    #[serde(rename = "active")]
    Active,

    #[serde(rename = "offline")]
    Offline,

    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct UserInstance {
    name: Option<String>,

    #[serde(rename = "softwareName")]
    software_name: Option<String>,

    #[serde(rename = "softwareVersion")]
    software_version: Option<String>,

    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,

    #[serde(rename = "faviconUrl")]
    favicon_url: Option<String>,

    #[serde(rename = "themeColor")]
    theme_color: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DriveFile {
    pub id: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "isSensitive")]
    pub is_sensitive: bool,

    pub name: String,

    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,

    pub url: String,

    #[serde(rename = "type")]
    pub type_: String,

    pub size: i64,

    pub md5: String,

    #[serde(rename = "blurhash")]
    pub blur_hash: Option<String>,

    pub comment: Option<String>,
    // TODO: properties: Record<string, any>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    #[serde(rename = "public")]
    Public,

    #[serde(rename = "home")]
    Home,

    #[serde(rename = "followers")]
    Followers,

    #[serde(rename = "specified")]
    Specified,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Poll {
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,

    pub multiple: bool,

    pub choices: Vec<PollChoice>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PollChoice {
    #[serde(rename = "isVoted")]
    pub is_voted: bool,

    pub text: String,

    pub votes: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EmojiSimple {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Emoji {
    pub id: String,

    pub aliases: Vec<String>,

    pub name: String,

    pub category: Option<String>,

    pub host: Option<String>,

    pub url: String,

    pub license: Option<String>,

    #[serde(rename = "isSensitive")]
    pub is_sensitive: bool,

    #[serde(rename = "localOnly")]
    pub local_only: bool,

    #[serde(rename = "roleIdsThatCanBeUsedThisEmojiAsReaction")]
    pub role_ids_that_can_be_used_this_emoji_as_reaction: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "body")]
#[non_exhaustive]
pub enum WsMsg {
    #[serde(rename = "channel")]
    Channel(WsMsgChannelBody),
    #[serde(rename = "noteUpdated")]
    NoteUpdated(NoteUpdatedBody),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum WsMsgChannelBody {
    #[serde(rename = "note")]
    Note { id: String, body: Note },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NoteUpdatedBody {
    #[serde(rename = "reacted")]
    NoteUpdatedBodyReacted {
        id: String,
        body: NoteUpdatedBodyReactedBody,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteUpdatedBodyReactedBody {
    pub emoji: Option<EmojiSimple>,

    pub reaction: String,

    #[serde(rename = "userId")]
    pub user_id: String,
}
