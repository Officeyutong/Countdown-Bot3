use super::{AbstractEvent, UnknownEvent};
use anyhow::anyhow;
use countdown_bot_proc_macro::impl_upcast;
use serde::Deserialize;
use serde_json::Value as JsonValue;
#[derive(Deserialize, Debug, Clone)]
#[impl_upcast(AbstractEvent)]
pub enum MessageEvent {
    Private(PrivateMessageEvent),
    Group(GroupMessageEvent),
    Guild(GuildMessageEvent),
    Unknown,
}

impl MessageEvent {
    pub fn from_json(
        json: &JsonValue,
    ) -> std::result::Result<MessageEvent, Box<dyn std::error::Error>> {
        if let JsonValue::Object(val) = json {
            return Ok(
                match val
                    .get("message_type")
                    .ok_or(anyhow!("Missing 'message_type'"))?
                    .as_str()
                    .ok_or("Expected string for 'message_type'")?
                {
                    "private" => MessageEvent::Private(serde_json::from_value::<
                        PrivateMessageEvent,
                    >(json.clone())?),
                    "group" => MessageEvent::Group(serde_json::from_value::<GroupMessageEvent>(
                        json.clone(),
                    )?),
                    "guild" => MessageEvent::Guild(serde_json::from_value::<GuildMessageEvent>(
                        json.clone(),
                    )?),

                    _ => MessageEvent::Unknown,
                },
            );
        } else {
            return Err(Box::from(anyhow!("Expected a JSON object!")));
        }
    }
}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SenderSex {
    Male,
    Female,
    Unknown,
}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivateMessageSubType {
    Friend,
    Group,
    Other,
}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct PrivateEventSender {
    pub user_id: Option<i64>,
    pub nickname: Option<String>,
    pub sex: Option<SenderSex>,
    pub age: Option<i32>,
}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct PrivateMessageEvent {
    // private
    pub message_type: String,
    // friend, group, other
    pub sub_type: PrivateMessageSubType,
    pub message_id: i64,
    pub user_id: i64,
    pub message: String,
    pub raw_message: String,
    pub font: i64,
    pub sender: PrivateEventSender,
}
impl AbstractEvent for PrivateMessageEvent {}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroupMessageSubType {
    Normal,
    Anonymous,
    Notice,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct GroupMessageEvent {
    pub message_type: String,
    pub sub_type: GroupMessageSubType,
    pub message_id: i64,
    pub group_id: i64,
    pub user_id: i64,
    pub anonymous: Option<AnonymousData>,
    pub message: String,
    pub raw_message: String,
    pub font: i64,
    pub sender: GroupMessageSender,
}
impl AbstractEvent for GroupMessageEvent {}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct AnonymousData {
    pub id: i64,
    pub name: String,
    pub flag: String,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroupSenderRole {
    Owner,
    Admin,
    Member,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct GroupMessageSender {
    pub user_id: Option<i64>,
    pub nickname: Option<String>,
    pub card: Option<String>,
    pub sex: Option<SenderSex>,
    pub age: Option<i32>,
    pub area: Option<String>,
    pub level: Option<String>,
    pub role: Option<GroupSenderRole>,
    pub title: Option<String>,
}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct GuildMessageEvent {
    pub sub_type: String,
    pub guild_id: String,
    pub channel_id: String,
    pub user_id: String,
    pub message_id: String,
    pub sender: GuildMessageSender,
    pub message: String,
}
impl AbstractEvent for GuildMessageEvent {}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct GuildMessageSender {
    pub user_id: Option<i64>,
    pub nickname: Option<String>,
    pub card: Option<String>,
    pub sex: Option<SenderSex>,
    pub age: Option<i32>,
    pub area: Option<String>,
    pub level: Option<String>,
    pub role: Option<GroupSenderRole>,
    pub title: Option<String>,
    pub tiny_id: String,
}
