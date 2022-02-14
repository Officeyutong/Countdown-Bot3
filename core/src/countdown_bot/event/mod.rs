pub mod manager;
pub mod message;
pub mod meta;
pub mod notice;
pub mod request;
use std::sync::Arc;

use anyhow::anyhow;
use downcast_rs::Downcast;
use message::MessageEvent;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use self::{meta::MetaEvent, notice::NoticeEvent, request::RequestEvent};

use super::client::ResultType;
#[derive(Deserialize, Debug, Clone)]
pub enum Event {
    Message(MessageEvent),
    Notice(NoticeEvent),
    Request(RequestEvent),
    Meta(MetaEvent),
    Unknown,
}
impl Event {
    pub fn perform_upcast(self) -> Arc<dyn AbstractEvent> {
        return match self {
            Event::Message(v) => v.to_event_trait(),
            Event::Notice(v) => v.to_event_trait(),
            Event::Request(v) => v.to_event_trait(),
            Event::Meta(v) => v.to_event_trait(),
            Event::Unknown => UnknownEvent::get_instance(),
        };
    }
}

#[derive(Debug, Clone)]
pub struct EventContainer {
    pub raw_value: Arc<JsonValue>,
    pub time: u64,
    pub self_id: u64,
    pub post_type: String,
    pub event: Event,
}

impl EventContainer {
    pub fn from_json_unknown(json: &JsonValue) -> ResultType<EventContainer> {
        #[derive(Deserialize)]
        struct LocalStruct {
            pub time: u64,
            pub self_id: u64,
            pub post_type: String,
        }
        let deser = serde_json::from_value::<LocalStruct>(json.clone())?;
        return Ok(EventContainer {
            time: deser.time,
            self_id: deser.self_id,
            post_type: deser.post_type,
            event: Event::Unknown,
            raw_value: Arc::new(json.clone()),
        });
    }
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
                raw_value: Arc::new(json.clone()),
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

pub trait AbstractEvent: Downcast + Sync + Send {}

downcast_rs::impl_downcast!(AbstractEvent);

pub type ConcreteEventWrapper = Arc<dyn AbstractEvent>;
pub struct OOPEventContainer {
    pub event: ConcreteEventWrapper,
    pub raw_value: Arc<JsonValue>,
    pub time: u64,
    pub self_id: u64,
    pub post_type: String,
}

pub struct UnknownEvent;
impl AbstractEvent for UnknownEvent {}

impl UnknownEvent {
    pub fn get_instance() -> ConcreteEventWrapper {
        return Arc::new(Self);
    }
}
