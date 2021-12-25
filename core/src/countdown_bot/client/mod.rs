use log::{debug, info};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use self::message::MessageIdResp;

use super::{command::SenderType, event::message::MessageEvent};

// pub type ReceiverMapWrapper = RefCell<ReceiverMap>;
pub type RequestReceiver = mpsc::UnboundedReceiver<APICallRequest>;
pub type RequestSender = mpsc::UnboundedSender<APICallRequest>;

pub type SenderContainer = std::result::Result<Value, Box<dyn std::error::Error + Send>>;
pub type SingleCallSender = oneshot::Sender<SenderContainer>;
mod message;
#[derive(Debug)]
pub struct APICallRequest {
    pub token: String,
    pub action: String,
    pub payload: Value,
    pub sender: SingleCallSender,
}
#[derive(Debug, Deserialize)]
pub struct APICallResponse {
    pub status: String,
    pub retcode: i32,
    pub data: Value,
    pub echo: String,
}

#[derive(Clone)]
pub struct CountdownBotClient {
    request_sender: RequestSender,
}
unsafe impl std::marker::Send for CountdownBotClient {}
impl CountdownBotClient {
    pub fn new(request_sender: RequestSender) -> CountdownBotClient {
        CountdownBotClient { request_sender }
    }
    pub async fn call(
        &self,
        action: &str,
        params: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let (tx, rx) = oneshot::channel::<SenderContainer>();
        let token = uuid::Uuid::new_v4().to_string();
        debug!("Performing api calling: {}, params: {}", action, params);
        self.request_sender.send(APICallRequest {
            action: String::from(action),
            payload: params.clone(),
            sender: tx,
            token: token.clone(),
        })?;
        match rx.await {
            Ok(o) => match o {
                Ok(o2) => return Ok(o2),
                Err(e) => return Err(e),
            },
            Err(e) => return Err(Box::new(e)),
        }
    }
}

impl CountdownBotClient {
    pub async fn quick_send(
        &self,
        evt: &MessageEvent,
        text: &String,
    ) -> Result<MessageIdResp, Box<dyn std::error::Error>> {
        match evt {
            MessageEvent::Private(evt) => {
                self.send_private_message(evt.sender.user_id.unwrap(), text, false)
                    .await
            }
            MessageEvent::Group(evt) => self.send_group_message(evt.group_id, text, false).await,
            MessageEvent::Unknown => Err(Box::from(anyhow::anyhow!("Invalid message event type"))),
        }
    }
    pub async fn quick_send_by_sender(
        &self,
        sender: &SenderType,
        text: &String,
    ) -> Result<MessageIdResp, Box<dyn std::error::Error>> {
        match sender {
            SenderType::Console(_) => {
                info!("{}", text);
                Ok(MessageIdResp { message_id: -1 })
            }
            SenderType::Private(evt) => self.quick_send(&MessageEvent::Private(evt.clone()), text).await,
            SenderType::Group(evt) => self.quick_send(&MessageEvent::Group(evt.clone()), text).await,
        }
    }
}
