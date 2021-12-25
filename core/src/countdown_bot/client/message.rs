use std::fmt::Display;

use crate::{countdown_bot::event::message::GroupMessageSender, declare_api_call};

use super::{CountdownBotClient, ResultType};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MessageIdResp {
    pub message_id: i32,
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
    pub time: i32,
    pub message_type: MessageInfoType,
    pub message_id: i32,
    pub real_id: i32,
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
    declare_api_call!(delete_message, (), (message_id, i32));
    pub async fn get_forward_message(&self, _id: &str) -> ResultType<MessageInfoResp> {
        todo!();
    }
}
