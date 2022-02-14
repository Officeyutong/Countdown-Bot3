use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::segment::MessageSegment;

impl<'de> Deserialize<'de> for MessageSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        #[derive(Deserialize)]
        struct LocalStruct {
            pub(crate) data: Value,
            pub(crate) r#type: String,
        }
        // let json_obj = serde_json::Value::deserialize(deserializer)?;
        let obj = LocalStruct::deserialize(deserializer).map_err(Error::custom)?;
        use serde_json::from_value;
        let v = obj.data;
        let parsed_data = match obj.r#type.as_str() {
            "text" => MessageSegment::Text(from_value(v).map_err(Error::custom)?),
            "face" => MessageSegment::Face(from_value(v).map_err(Error::custom)?),
            "image" => MessageSegment::Image(from_value(v).map_err(Error::custom)?),
            "record" => MessageSegment::Record(from_value(v).map_err(Error::custom)?),
            "video" => MessageSegment::Video(from_value(v).map_err(Error::custom)?),
            "rps" => MessageSegment::RPS,
            "dice" => MessageSegment::Dice,
            "shake" => MessageSegment::Shake,
            "poke" => MessageSegment::Poke(from_value(v).map_err(Error::custom)?),
            "anonymous" => MessageSegment::Anonymous,
            "share" => MessageSegment::Share(from_value(v).map_err(Error::custom)?),
            "contact" => MessageSegment::Contact(from_value(v).map_err(Error::custom)?),
            "location" => MessageSegment::Location(from_value(v).map_err(Error::custom)?),
            "music" => MessageSegment::Music(from_value(v).map_err(Error::custom)?),
            "reply" => MessageSegment::Reply(from_value(v).map_err(Error::custom)?),
            "forward" => MessageSegment::Forward(from_value(v).map_err(Error::custom)?),
            "node" => MessageSegment::Node(from_value(v).map_err(Error::custom)?),
            "xml" => MessageSegment::XML(from_value(v).map_err(Error::custom)?),
            "json" => MessageSegment::JSON(from_value(v).map_err(Error::custom)?),
            "cardimage" => MessageSegment::CardImage(from_value(v).map_err(Error::custom)?),
            "tts" => MessageSegment::TTS(from_value(v).map_err(Error::custom)?),
            tp => {
                return Err(Error::custom(format!(
                    "Unknown message segment type: {}",
                    tp
                )))
            }
        };
        return Ok(parsed_data);
    }
}
impl Serialize for MessageSegment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        use serde::ser::SerializeMap;
        // let json_val = serde_json::Value::serialize(&self, serializer)?;
        let (r#type, data) = match self {
            MessageSegment::Text(v) => ("text", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::Face(v) => ("face", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::Record(v) => {
                ("record", serde_json::to_value(v).map_err(Error::custom)?)
            }
            MessageSegment::Video(v) => ("video", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::At(v) => ("at", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::RPS => ("rps", Value::Object(Default::default())),
            MessageSegment::Dice => ("dice", Value::Object(Default::default())),
            MessageSegment::Shake => ("shake", Value::Object(Default::default())),
            MessageSegment::Anonymous => ("anonymous", Value::Object(Default::default())),
            MessageSegment::Share(v) => ("share", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::Contact(v) => {
                ("contact", serde_json::to_value(v).map_err(Error::custom)?)
            }
            MessageSegment::Location(v) => {
                ("location", serde_json::to_value(v).map_err(Error::custom)?)
            }
            MessageSegment::Music(v) => ("music", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::Image(v) => ("image", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::Reply(v) => ("reply", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::Redbag(v) => {
                ("redbag", serde_json::to_value(v).map_err(Error::custom)?)
            }
            MessageSegment::Poke(v) => ("poke", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::Gift(v) => ("gift", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::Forward(v) => {
                ("forward", serde_json::to_value(v).map_err(Error::custom)?)
            }
            MessageSegment::Node(v) => ("node", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::XML(v) => ("xml", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::JSON(v) => ("json", serde_json::to_value(v).map_err(Error::custom)?),
            MessageSegment::CardImage(v) => {
                ("cardimage", serde_json::to_value(v).map_err(Error::custom)?)
            }
            MessageSegment::TTS(v) => ("tts", serde_json::to_value(v).map_err(Error::custom)?),
        };
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", r#type)?;
        map.serialize_entry("data", &data)?;
        return map.end();
    }
}

impl ToString for MessageSegment {
    fn to_string(&self) -> String {
        match self {
            MessageSegment::Text(v) => v.to_string(),
            MessageSegment::Face(v) => v.to_string(),
            MessageSegment::Record(v) => v.to_string(),
            MessageSegment::Video(v) => v.to_string(),
            MessageSegment::At(v) => v.to_string(),
            MessageSegment::RPS => "[CQ:rps]".to_string(),
            MessageSegment::Dice => "[CQ:dice]".to_string(),
            MessageSegment::Shake => "[CQ:shake]".to_string(),
            MessageSegment::Anonymous => "[CQ:anonymous]".to_string(),
            MessageSegment::Share(v) => v.to_string(),
            MessageSegment::Contact(v) => v.to_string(),
            MessageSegment::Location(v) => v.to_string(),
            MessageSegment::Music(v) => v.to_string(),
            MessageSegment::Image(v) => v.to_string(),
            MessageSegment::Reply(v) => v.to_string(),
            MessageSegment::Redbag(v) => v.to_string(),
            MessageSegment::Poke(v) => v.to_string(),
            MessageSegment::Gift(v) => v.to_string(),
            MessageSegment::Forward(v) => v.to_string(),
            MessageSegment::Node(v) => v.to_string(),
            MessageSegment::XML(v) => v.to_string(),
            MessageSegment::JSON(v) => v.to_string(),
            MessageSegment::CardImage(v) => v.to_string(),
            MessageSegment::TTS(v) => v.to_string(),
        }
        .clone()
    }
}
