use std::error::Error;

use anyhow::anyhow;
use countdown_bot_proc_macro::impl_upcast;
use serde::Deserialize;
use serde_json::{from_value, Value};

use super::AbstractEvent;
use super::UnknownEvent;
#[derive(Deserialize, Debug, Clone)]
#[impl_upcast(AbstractEvent)]
pub enum MetaEvent {
    Lifecycle(LifecycleEvent),
    Heartbeat(HeartbeatEvent),
    Unknown,
}

impl MetaEvent {
    pub fn from_json(json: &Value) -> Result<MetaEvent, Box<dyn Error>> {
        if let Value::Object(val) = json {
            let t = json.clone();
            return Ok(
                match val
                    .get("meta_event_type")
                    .ok_or(anyhow!("Missing 'meta_event_type'"))?
                    .as_str()
                    .ok_or("Expected string for 'meta_event_type'")?
                {
                    "heartbeat" => MetaEvent::Heartbeat(from_value::<HeartbeatEvent>(t)?),
                    "lifecycle" => MetaEvent::Lifecycle(from_value::<LifecycleEvent>(t)?),
                    _ => MetaEvent::Unknown,
                },
            );
        } else {
            return Err(Box::from(anyhow!("Expected JSON object!")));
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct HeartbeatEvent {
    pub interval: i64,
    pub status: Value,
}
impl AbstractEvent for HeartbeatEvent {}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEventSubType {
    Enable,
    Disable,
    Connect,
}
#[derive(Deserialize, Debug, Clone)]
pub struct LifecycleEvent {
    pub sub_type: LifecycleEventSubType,
}
impl AbstractEvent for LifecycleEvent {}
