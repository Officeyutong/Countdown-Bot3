use self::message::MessageIdResp;
use log::{debug, info};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use super::{command::SenderType, event::message::MessageEvent};

pub type RequestReceiver = mpsc::UnboundedReceiver<APICallRequest>;
pub type RequestSender = mpsc::UnboundedSender<APICallRequest>;

pub type SenderContainer = std::result::Result<Value, Box<dyn std::error::Error + Send>>;
pub type SingleCallSender = oneshot::Sender<SenderContainer>;
pub type ResultType<T> = Result<T, Box<dyn std::error::Error>>;
pub type ResultSendType<T> = Result<T, Box<dyn std::error::Error + Send>>;
pub fn create_result<T: DeserializeOwned>(resp: ResultType<Value>) -> ResultType<T> {
    return match resp {
        Ok(o) => Ok(serde_json::from_value::<T>(o)?),
        Err(e) => Err(e),
    };
}

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
        debug!("Performing async api call: {}, params: {}", action, {
            let s = params.to_string();
            s[..s.len().min(1000)].to_string()
        });
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
    pub fn sync_call(
        &self,
        action: &str,
        params: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let (tx, rx) = oneshot::channel::<SenderContainer>();
        let token = uuid::Uuid::new_v4().to_string();
        debug!("Performing sync api call: {}, params: {}", action, {
            let s = params.to_string();
            s[..s.len().min(1000)].to_string()
        });
        self.request_sender.send(APICallRequest {
            action: String::from(action),
            payload: params.clone(),
            sender: tx,
            token: token.clone(),
        })?;
        match rx.blocking_recv() {
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
        text: &str,
    ) -> Result<MessageIdResp, Box<dyn std::error::Error>> {
        match evt {
            MessageEvent::Private(evt) => {
                self.send_private_msg(evt.sender.user_id.unwrap(), text, false)
                    .await
            }
            MessageEvent::Group(evt) => self.send_group_msg(evt.group_id, text, false).await,
            MessageEvent::Unknown => Err(Box::from(anyhow::anyhow!("Invalid message event type"))),
        }
    }
    pub fn quick_send_sync(
        &self,
        evt: &MessageEvent,
        text: &str,
    ) -> Result<MessageIdResp, Box<dyn std::error::Error>> {
        match evt {
            MessageEvent::Private(evt) => {
                self.send_private_msg_sync(evt.sender.user_id.unwrap(), text, false)
            }
            MessageEvent::Group(evt) => self.send_group_msg_sync(evt.group_id, text, false),
            MessageEvent::Unknown => Err(Box::from(anyhow::anyhow!("Invalid message event type"))),
        }
    }
    pub async fn quick_send_by_sender(
        &self,
        sender: &SenderType,
        text: &str,
    ) -> Result<MessageIdResp, Box<dyn std::error::Error>> {
        match sender {
            SenderType::Console(_) => {
                info!("{}", text);
                Ok(MessageIdResp { message_id: -1 })
            }
            SenderType::Private(evt) => {
                self.quick_send(&MessageEvent::Private(evt.clone()), text)
                    .await
            }
            SenderType::Group(evt) => {
                self.quick_send(&MessageEvent::Group(evt.clone()), text)
                    .await
            }
        }
    }
    pub fn quick_send_by_sender_sync(
        &self,
        sender: &SenderType,
        text: &str,
    ) -> Result<MessageIdResp, Box<dyn std::error::Error>> {
        match sender {
            SenderType::Console(_) => {
                info!("{}", text);
                Ok(MessageIdResp { message_id: -1 })
            }
            SenderType::Private(evt) => {
                self.quick_send_sync(&MessageEvent::Private(evt.clone()), text)
            }
            SenderType::Group(evt) => self.quick_send_sync(&MessageEvent::Group(evt.clone()), text),
        }
    }
}

mod group;
mod message;
mod misc;
#[macro_export]
macro_rules! declare_api_call {
    ($name:ident,$ret:ty, $(($x:ident,$y:ty)),*) => {
        pub async fn $name(
            &self,
            $($x:$y,)*
        )->$crate::countdown_bot::client::ResultType<$ret> {
            $crate::countdown_bot::client::create_result(
                self.call(
                    stringify!($name),
                    &serde_json::json!({
                        $(stringify!($x):$x,)*
                    })
                ).await
            )
        }
        paste::paste! {
            pub fn [<$name _sync>] (
                &self,
                $($x:$y,)*
            )->$crate::countdown_bot::client::ResultType<$ret> {
                $crate::countdown_bot::client::create_result(
                    self.sync_call(
                        stringify!($name),
                        &serde_json::json!({
                            $(stringify!($x):$x,)*
                        })
                    )
                )
            }
        }
    };
}
