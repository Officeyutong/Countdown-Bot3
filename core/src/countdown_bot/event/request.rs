use std::error::Error;

use anyhow::anyhow;
use countdown_bot_proc_macro::impl_upcast;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};

use super::AbstractEvent;
use super::UnknownEvent;
#[derive(Deserialize, Debug, Clone)]
#[impl_upcast(AbstractEvent)]
pub enum RequestEvent {
    FriendRequest(FriendRequestEvent),
    GroupRequest(GroupRequestEvent),
    Unknown,
}

impl RequestEvent {
    pub fn from_json(json: &Value) -> Result<RequestEvent, Box<dyn Error>> {
        if let Value::Object(val) = json {
            let t = json.clone();
            return Ok(
                match val
                    .get("request_type")
                    .ok_or(anyhow!("Missing 'request_type'"))?
                    .as_str()
                    .ok_or("Expected string for 'request_type'")?
                {
                    "friend" => RequestEvent::FriendRequest(from_value::<FriendRequestEvent>(t)?),
                    "group" => RequestEvent::GroupRequest(from_value::<GroupRequestEvent>(t)?),
                    _ => RequestEvent::Unknown,
                },
            );
        } else {
            return Err(Box::from(anyhow!("Expected JSON object!")));
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FriendRequestEvent {
    pub user_id: i64,
    pub comment: String,
    pub flag: String,
}
impl AbstractEvent for FriendRequestEvent {}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GroupRequestSubType {
    Add,
    Invite,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupRequestEvent {
    pub sub_type: GroupRequestSubType,
    pub group_id: i64,
    pub user_id: i64,
    pub comment: String,
    pub flag: String,
}
impl AbstractEvent for GroupRequestEvent {}
