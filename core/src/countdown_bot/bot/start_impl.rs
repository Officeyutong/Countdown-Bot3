use std::collections::HashMap;

use super::CountdownBot;
use crate::countdown_bot::bot::ReceiverMap;
use crate::countdown_bot::client::{APICallRequest, APICallResponse, CountdownBotClient};
use crate::countdown_bot::command::{CommandSender, ConsoleSender};
use crate::countdown_bot::event::EventContainer;
use crate::countdown_bot::plugin::PluginWrapperArc;
use anyhow::anyhow;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, trace};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

impl CountdownBot {
    pub async fn run(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use tokio_tungstenite::connect_async;
        use url::Url;
        let url_event = {
            let mut local = Url::parse(&self.config.server_url)?.join("event").unwrap();
            local.set_query(Some(
                format!("access_token={}", self.config.access_token).as_str(),
            ));
            local
        };
        let url_call = {
            let mut local = Url::parse(&self.config.server_url)?.join("api").unwrap();
            local.set_query(Some(
                format!("access_token={}", self.config.access_token).as_str(),
            ));
            local
        };
        let (call_tx, mut call_rx) = mpsc::unbounded_channel::<APICallRequest>();
        fn construct_json(action: String, params: Value, token: String) -> String {
            return serde_json::to_string(&serde_json::json!({
                "action": action,
                "params": params,
                "echo": token
            }))
            .unwrap();
        }
        let (stop_tx, stop_rx) = tokio::sync::watch::channel::<bool>(false);
        self.stop_signal_sender = Some(stop_tx);
        self.stop_signal_receiver = Some(stop_rx.clone());
        self.schedule_loop_manager
            .set_stop_signal_receiver(stop_rx.clone());
        let (console_tx, mut console_rx) = mpsc::unbounded_channel::<String>();
        {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut stop_rx = self.stop_signal_receiver.as_ref().unwrap().clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(tokio::io::stdin()).lines();
                loop {
                    tokio::select! {
                        _ = stop_rx.changed() => {
                            if *stop_rx.borrow() {
                                info!("Shutting down console reader..");
                                break;
                            }
                        }
                        Ok(Some(line)) = reader.next_line() => {
                            if !line.is_empty() {
                                console_tx.send(line).ok();
                            }
                        }

                    }
                }
            });
        }

        self.client = Some(CountdownBotClient::new(call_tx.clone()));
        {
            for (name, wrapper) in self
                .plugin_manager
                .plugins
                .iter()
                .map(|(x, y)| (x.clone(), y.clone()))
                .collect::<Vec<(String, PluginWrapperArc)>>()
            {
                self.state_manager.set_curr_plugin(name.clone());
                self.command_manager.update_plugin_name(name.clone());
                self.schedule_loop_manager
                    .set_current_plugin(wrapper.plugin_instance.clone());
                wrapper
                    .plugin_instance
                    .lock()
                    .await
                    .on_before_start(self, self.create_client())?;
            }
        }
        {
            let loop_manager = self.schedule_loop_manager.clone();
            tokio::spawn(loop_manager.run());
        }
        {
            let local_stop_rx = stop_rx.clone();
            let local_cfg = self.config.clone();
            tokio::spawn(async move {
                let mut stop_rx = local_stop_rx;
                let mut receiver_map: ReceiverMap = HashMap::new();
                let cfg = local_cfg;
                loop {
                    match connect_async(url_call.clone()).await {
                        Ok((stream, _resp)) => {
                            info!("API handler connected.");
                            let (mut write, mut read) = stream.split();
                            loop {
                                tokio::select! {
                                    _   = stop_rx.changed() => {
                                        if *stop_rx.borrow() {
                                            info!("Shutting down API handler..");
                                            return;
                                        }
                                    }
                                    Some(result) = read.next() => {
                                        let json = serde_json::from_str::<serde_json::Value>(
                                            &result.unwrap().to_string().as_str(),
                                        ).unwrap();
                                        if let Ok(parse_result) = serde_json::from_value::<APICallResponse>(json.clone()) {
                                            if let Some(sender) = receiver_map.remove(&parse_result.echo) {
                                                sender.send(match parse_result.status.as_str() {
                                                    "ok" => Ok(parse_result.data),
                                                    "failed" => Err(Box::from(anyhow!(
                                                        "Failed to perform API call: {}",
                                                        parse_result.retcode
                                                    ))),
                                                    "async" => Ok(serde_json::json!({})),
                                                    _ => Err(Box::from(anyhow!(
                                                        "Invalid status: {}",
                                                        parse_result.status
                                                    ))),
                                                }).ok();
                                            }
                                        } else {
                                            error!("Invalid call response: {:?}", &json);
                                        }
                                    }
                                    call_req = call_rx.recv() => {
                                        if let Some(req) = call_req{
                                            receiver_map.insert(req.token.clone(), req.sender);
                                            if let Err(err) = write
                                                .send(Message::Text(construct_json(
                                                    req.action.clone(),
                                                    req.payload.clone(),
                                                    req.token.clone(),
                                                )))
                                                .await
                                            {
                                                if let Some(r) = receiver_map.remove(&req.token){
                                                    if let Ok(_) = r.send(Err(Box::from(anyhow!("Sending error! {}", err)))){

                                                    }
                                                }
                                            }
                                        }
                                    }
                                };
                            }
                        }
                        Err(err) => {
                            error!("Error occurred: {}", err);
                            info!("Reconnecting after {} seconds..", cfg.reconnect_interval);
                            tokio::time::sleep(core::time::Duration::from_secs(
                                cfg.reconnect_interval.into(),
                            ))
                            .await;
                        }
                    }
                }
            });
        }
        while !self.stop {
            match connect_async(url_event.clone()).await {
                Ok((stream, resp)) => {
                    info!("Event handler connected! {}", resp.status());
                    let (write, read) = stream.split();
                    self.write_stream = Some(write);
                    self.read_stream = Some(read);
                    loop {
                        let mut stop_rx = self.stop_signal_receiver.as_ref().unwrap().clone();
                        trace!("Selecting..");
                        tokio::select! {
                            line = console_rx.recv() => {
                                self.dispatch_command(CommandSender::Console(ConsoleSender { line: line.unwrap() } )).await;
                            }
                            signal_result = tokio::signal::ctrl_c() => {
                                if let Ok(_) = signal_result {
                                    self.stop_signal_sender.as_ref().unwrap().clone().send(true).expect("?");
                                }
                            }
                            _   = stop_rx.changed() => {
                                if *stop_rx.borrow() {
                                    self.shutdown().await;
                                }
                            }
                            Some(result) = self.read_stream.as_mut().unwrap().next() => {
                                match result {
                                    Ok(message) => {
                                        let raw_string = message.to_string();
                                        match serde_json::from_str::<serde_json::Value>(
                                            &raw_string.as_str(),
                                        ) {
                                            Ok(json) => {
                                                match EventContainer::from_json(&json) {
                                                    Ok(event) => {self.dispatch_event(&event).await;}
                                                    Err(e) => error!("Malformed event object: {}\n{}", e, json)
                                                }
                                            }
                                            Err(err) => {
                                                error!("Invalid json! {}", err);
                                                break;
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        error!("Error occurred: {}", err);
                                        break;
                                    }
                                }
                            }

                        };
                    }
                }
                Err(err) => {
                    error!("Error occurred: {}", err);
                    info!(
                        "Reconnecting after {} seconds..",
                        self.config.reconnect_interval
                    );
                    tokio::time::sleep(core::time::Duration::from_secs(
                        self.config.reconnect_interval.into(),
                    ))
                    .await;
                }
            }
        }

        return Ok(());
    }
}
