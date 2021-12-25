use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value as JsonValue;
#[derive(Deserialize, Debug, Clone)]
pub enum MessageEvent {
    Private(PrivateMessageEvent),
    Group(GroupMessageEvent),
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
                    _ => MessageEvent::Unknown,
                },
            );
        } else {
            return Err(Box::from(anyhow!("Expected a JSON object!")));
        }
    }
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SenderSex {
    Male,
    Female,
    Unknown,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PrivateMessageSubType {
    Friend,
    Group,
    Other,
}
#[derive(Deserialize, Debug, Clone)]
pub struct PrivateEventSender {
    pub user_id: Option<i64>,
    pub nickname: Option<String>,
    pub sex: Option<SenderSex>,
    pub age: Option<i32>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct PrivateMessageEvent {
    // private
    pub message_type: String,
    // friend, group, other
    pub sub_type: PrivateMessageSubType,
    pub message_id: i32,
    pub user_id: i32,
    pub message: String,
    pub raw_message: String,
    pub font: i32,
    pub sender: PrivateEventSender,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GroupMessageSubType {
    Normal,
    Anonymous,
    Notice,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GroupMessageEvent {
    pub message_type: String,
    pub sub_type: GroupMessageSubType,
    pub message_id: i32,
    pub group_id: i64,
    pub user_id: i32,
    pub anonymous: Option<AnonymousData>,
    pub message: String,
    pub raw_message: String,
    pub font: i32,
    pub sender: GroupMessageSender,
}
#[derive(Deserialize, Debug, Clone)]
pub struct AnonymousData {
    pub id: i64,
    pub name: String,
    pub flag: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GroupSenderRole {
    Owner,
    Admin,
    Member,
}

#[derive(Deserialize, Debug, Clone)]
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
