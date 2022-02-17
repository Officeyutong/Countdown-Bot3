use std::fmt::Display;

use crate::{
    countdown_bot::{event::message::GroupMessageSender, message::wrapper::Message},
    declare_api_call,
};

use super::{CountdownBotClient, ResultType};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct MessageIdResp {
    pub message_id: i64,
}
#[derive(Deserialize, Debug)]
pub struct ComposedMessageId {
    pub message_id_i64: i64,
    pub message_id_str: String,
}
impl Into<ComposedMessageId> for MessageIdResp {
    fn into(self) -> ComposedMessageId {
        return ComposedMessageId {
            message_id_i64: self.message_id,
            message_id_str: String::new(),
        };
    }
}
impl Display for MessageIdResp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("MessageIdResp {{message_id={}}}", self.message_id).as_str())?;
        Ok(())
    }
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MessageInfoType {
    Private,
    Group,
}
#[derive(Deserialize, Debug)]
pub struct MessageInfoResp {
    pub time: i64,
    pub message_type: MessageInfoType,
    pub message_id: i64,
    pub real_id: i64,
    pub sender: GroupMessageSender,
    pub message: String,
}

impl CountdownBotClient {
    declare_api_call!(
        send_private_msg,
        MessageIdResp,
        (user_id, i64),
        (message, &str),
        (auto_escape, bool)
    );
    declare_api_call!(
        send_group_msg,
        MessageIdResp,
        (group_id, i64),
        (message, &str),
        (auto_escape, bool)
    );
    declare_api_call!(delete_message, (), (message_id, i64));
    pub async fn msgseg_send_private_msg(
        &self,
        uid: i64,
        message: &Message,
    ) -> ResultType<MessageIdResp> {
        match message {
            Message::Text(t) => return Ok(self.send_private_msg(uid, &t, false).await?),
            Message::Segment(seg) => {
                let json_val = serde_json::to_value(seg)?;
                return Ok(serde_json::from_value(
                    self.call(
                        "send_private_msg",
                        &json!({
                            "user_id":uid,
                            "message":json_val,
                        }),
                    )
                    .await?,
                )?);
            }
        };
    }
    pub async fn msgseg_send_group_msg(
        &self,
        gid: i64,
        message: &Message,
    ) -> ResultType<MessageIdResp> {
        match message {
            Message::Text(t) => return Ok(self.send_group_msg(gid, &t, false).await?),
            Message::Segment(seg) => {
                let json_val = serde_json::to_value(seg)?;
                return Ok(serde_json::from_value(
                    self.call(
                        "send_group_msg",
                        &json!({
                            "group_id":gid,
                            "message":json_val,
                        }),
                    )
                    .await?,
                )?);
            }
        };
    }
}
