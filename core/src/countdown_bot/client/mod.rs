use log::debug;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use super::event::message::MessageEvent;

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
    pub async fn quick_send(&self, evt: &MessageEvent, _text: &String) {
        match evt {
            MessageEvent::Private(_) => {}
            MessageEvent::Group(_) => {}
            MessageEvent::Unknown => {}
        }
    }
}
