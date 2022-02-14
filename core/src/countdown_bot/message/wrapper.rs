use log::debug;
use serde::{de::Error, Deserialize};
use serde_json::Value;

use super::segment::MessageSegment;
#[derive(Debug, Clone)]
pub enum Message {
    Text(String),
    Segment(Vec<MessageSegment>),
}
impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let jsonval = Value::deserialize(deserializer)?;
        return match &jsonval {
            Value::String(s) => Ok(Message::Text(s.clone())),
            Value::Array(v) => Ok(Message::Segment({
                let mut out = vec![];
                for t in v.iter() {
                    out.push(serde_json::from_value(t.clone()).map_err(Error::custom)?);
                }
                out
            })),
            _ => Err(Error::custom("Invalid type")),
        };
    }
}
impl ToString for Message {
    fn to_string(&self) -> String {
        let output = match self {
            Message::Text(v) => v.clone(),
            Message::Segment(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(""),
        };
        debug!("Transformed to: {}, from = \n{:#?}", output, self);
        return output;
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}
impl From<Vec<MessageSegment>> for Message {
    fn from(s: Vec<MessageSegment>) -> Self {
        Self::Segment(s)
    }
}
