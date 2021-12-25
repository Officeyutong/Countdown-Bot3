use std::fmt::Display;

use super::CountdownBotClient;
use serde::Deserialize;
use serde_json::json;
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
impl CountdownBotClient {
    pub async fn send_private_message(
        &self,
        user_id: i64,
        message: &String,
        auto_escape: bool,
    ) -> Result<MessageIdResp, Box<dyn std::error::Error>> {
        let resp = self
            .call(
                "send_private_msg",
                &json!({
                    "user_id": user_id,
                    "message": message.clone(),
                    "auto_escape": auto_escape
                }),
            )
            .await;
        return match resp {
            Ok(o) => Ok(serde_json::from_value::<MessageIdResp>(o)?),
            Err(e) => Err(e),
        };
    }
}
