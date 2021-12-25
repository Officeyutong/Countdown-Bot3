pub mod handle;
pub mod message;
pub mod meta;
pub mod notice;
pub mod request;
use anyhow::anyhow;
use message::MessageEvent;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use self::{meta::MetaEvent, notice::NoticeEvent, request::RequestEvent};
#[derive(Deserialize, Debug, Clone)]
pub enum Event {
    Message(MessageEvent),
    Notice(NoticeEvent),
    Request(RequestEvent),
    Meta(MetaEvent),
    Unknown,
}
#[derive(Deserialize, Debug, Clone)]
pub struct EventContainer {
    pub raw_value: JsonValue,
    pub time: u64,
    pub self_id: u64,
    pub post_type: String,
    pub event: Event,
}

impl EventContainer {
    pub fn from_json(
        json: &JsonValue,
    ) -> std::result::Result<EventContainer, Box<dyn std::error::Error>> {
        if let JsonValue::Object(val) = json {
            let post_type = val
                .get("post_type")
                .ok_or(anyhow!("Missing 'post_type' field"))?
                .as_str()
                .ok_or("Expected string for 'post_type'")?;
            return Ok(EventContainer {
                raw_value: json.clone(),
                time: val
                    .get("time")
                    .ok_or(anyhow!("Missing 'time' field"))?
                    .as_u64()
                    .ok_or(anyhow!("Expected u64 for 'time' field"))?,
                self_id: val
                    .get("self_id")
                    .ok_or(anyhow!("Missing 'self_id' field"))?
                    .as_u64()
                    .ok_or(anyhow!("Expected u64 for 'self_id' field"))?,
                post_type: String::from(post_type),
                event: match post_type {
                    "message" => Event::Message(MessageEvent::from_json(json)?),
                    "notice" => Event::Notice(NoticeEvent::from_json(json)?),
                    "request" => Event::Request(RequestEvent::from_json(json)?),
                    "meta_event" => Event::Meta(MetaEvent::from_json(json)?),
                    _ => Event::Unknown,
                },
            });
        } else {
            return Err(Box::from(anyhow::anyhow!("Expected a JSON object!")));
        }
    }
}
