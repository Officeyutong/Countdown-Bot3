use std::sync::Arc;

use log::error;

use crate::countdown_bot::{
    command::{Command, CommandSender, SenderType},
    event::{message::MessageEvent, Event, EventContainer},
};

use super::CountdownBot;

impl CountdownBot {
    pub async fn dispatch_event(&mut self, event: &EventContainer) {
        if let Event::Message(ref msg_evt) = event.event {
            let msg_line = match msg_evt {
                MessageEvent::Private(e) => &e.message,
                MessageEvent::Group(e) => &e.message,
                MessageEvent::Unknown => return,
            };
            let mut ok_for_command = false;
            for prefix in self.config.command_prefix.iter() {
                if msg_line.starts_with(prefix.as_str()) {
                    ok_for_command = true;
                }
            }
            if ok_for_command {
                self.dispatch_command(CommandSender::User(event.clone()))
                    .await;
                return;
            }
        }
        for (_, val) in self.plugin_manager.plugins.iter() {
            val.plugin_instance
                .clone()
                .lock()
                .await
                .on_event(event.clone())
                .await;
        }
    }
    pub async fn dispatch_command(&mut self, sender: CommandSender) {
        let parsed_sender = sender.parse_sender().unwrap();
        let mut is_console = false;
        let enable_checker: fn(&Arc<Command>) -> bool = match &parsed_sender {
            SenderType::Console(_) => {
                is_console = true;
                |v| v.console_enabled
            }
            SenderType::Private(_) => |v| v.private_enabled,
            SenderType::Group(_) => |v| v.group_enabled,
        };
        let mut cmd_line = match &parsed_sender {
            SenderType::Console(evt) => evt.line.clone(),
            SenderType::Private(evt) => evt.message.clone(),
            SenderType::Group(evt) => evt.message.clone(),
        };
        for prefix in self.config.command_prefix.iter() {
            if cmd_line.starts_with(prefix.as_str()) {
                cmd_line = cmd_line.as_str()[prefix.len()..].to_string();
                break;
            }
        }
        let splitted = cmd_line.split(" ").collect::<Vec<&str>>();
        let exec_ret: Result<(), String> =
            match self.command_manager.get_command(&String::from(splitted[0])) {
                Ok(cmd) => {
                    if enable_checker(&cmd) {
                        if cmd.plugin_name.as_ref().unwrap() == "<bot>" {
                            let call_result = self
                                .on_command(
                                    cmd.command_name.clone(),
                                    splitted
                                        .iter()
                                        .map(|x| String::from(*x))
                                        .collect::<Vec<String>>(),
                                    parsed_sender.clone(),
                                )
                                .await;
                            if let Err(e) = call_result {
                                self.create_client()
                                    .quick_send_by_sender(
                                        &parsed_sender,
                                        format!("执行指令时发生错误:\n{}", e).as_str(),
                                    )
                                    .await
                                    .ok();
                                error!("{:#?}", e);
                            }
                        } else {
                            let cmd_local = cmd.clone();
                            let plugin = (self)
                                .plugin_manager
                                .plugins
                                .get(cmd_local.plugin_name.as_ref().unwrap())
                                .unwrap()
                                .clone()
                                .plugin_instance
                                .clone();
                            let cmd_name = cmd.command_name.clone();
                            let args = splitted
                                .iter()
                                .map(|x| String::from(*x))
                                .collect::<Vec<String>>();
                            let sender_cloned = parsed_sender.clone();
                            let client_cloned = self.create_client();
                            tokio::spawn(async move {
                                let local_sender = sender_cloned;
                                let call_ret = plugin
                                    .lock()
                                    .await
                                    .on_command(cmd_name, args, &local_sender)
                                    .await;
                                if let Err(e) = call_ret {
                                    error!("{:#?}", e);
                                    client_cloned
                                        .quick_send_by_sender(
                                            &local_sender,
                                            format!("执行指令时发生错误:\n{}", &e).as_str(),
                                        )
                                        .await
                                        .ok();
                                }
                            });
                        }
                        Ok(())
                    } else {
                        if is_console {
                            Err(String::from("This command does not support console"))
                        } else {
                            Err(String::from("此指令不支持当前对话环境"))
                        }
                    }
                }
                Err(err) => {
                    if is_console {
                        Err(String::from(format!("{}", err)))
                    } else {
                        Err(String::from(format!(
                            "指令不存在，请发送\"{}help\"来查看帮助!",
                            self.config.command_prefix[0]
                        )))
                    }
                }
            };
        let client = self.create_client();
        if let Err(s) = exec_ret {
            match &parsed_sender {
                SenderType::Console(_) => error!("{}", s),
                SenderType::Private(evt) => {
                    client
                        .quick_send(&MessageEvent::Private(evt.clone()), &s)
                        .await
                        .ok();
                }
                SenderType::Group(evt) => {
                    client
                        .quick_send(&MessageEvent::Group(evt.clone()), &s)
                        .await
                        .ok();
                }
            }
        }
    }
}
