use std::sync::Arc;

use crate::countdown_bot::{
    command::{Command, CommandSender, SenderType},
    event::{message::MessageEvent, Event, EventContainer, OOPEventContainer},
};
use anyhow::anyhow;
use log::{error, info, trace};
use tokio::sync::RwLock;

use super::CountdownBot;

impl CountdownBot {
    pub async fn dispatch_event(&mut self, event: EventContainer) {
        if let Event::Message(ref msg_evt) = event.event {
            let (msg_line, sender) = match msg_evt {
                MessageEvent::Private(e) => (e.raw_message.clone(), SenderType::Private(e.clone())),
                MessageEvent::Group(e) => (e.raw_message.clone(), SenderType::Group(e.clone())),
                MessageEvent::Guild(e) => (e.message.to_string(), SenderType::Guild(e.clone())),
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
            info!(
                "Message<{}>: {}",
                sender.generate_sender_message(),
                msg_line
            );
        }
        let oop_event = OOPEventContainer {
            event: event.event.perform_upcast(),
            raw_value: event.raw_value,
            time: event.time,
            self_id: event.self_id,
            post_type: event.post_type,
        };
        self.event_manager
            .dispatch_event(Arc::new(RwLock::new(oop_event)))
            .await;
        // for (_, val) in self.plugin_manager.plugins.iter() {
        //     let plugin_instance_ref = val.read().await.plugin_instance.clone();
        //     let event_cloned = event.clone();
        //     tokio::spawn(async move {
        //         let resp = plugin_instance_ref
        //             .write()
        //             .await
        //             .on_event(event_cloned.clone())
        //             .await;
        //         if let Err(e) = resp {
        //             error!(
        //                 "Error occured when dispatching event {:?}:\n{}",
        //                 event_cloned, e
        //             );
        //         }
        //     });
        //     // .await;
        // }
    }
    pub fn in_command_blacklist_check(&self, sender: &SenderType) -> bool {
        let user_id = match sender {
            SenderType::Console(_) => return false,
            SenderType::Private(v) => v.user_id,
            SenderType::Group(v) => v.user_id,
            SenderType::Guild(_) => return false,
        };
        return self.config.blacklist_users.contains(&(user_id as i64));
    }
    pub async fn dispatch_command(&mut self, sender: CommandSender) {
        let parsed_sender = sender.parse_sender().unwrap();
        if self.in_command_blacklist_check(&parsed_sender) {
            info!(
                "Ignoring command call from: {}",
                parsed_sender.generate_identifier()
            );
            return;
        }
        let mut is_console = false;
        let enable_checker: fn(&Arc<Command>) -> bool = match &parsed_sender {
            SenderType::Console(_) => {
                is_console = true;
                |v| v.console_enabled
            }
            SenderType::Private(_) => |v| v.private_enabled,
            SenderType::Group(_) => |v| v.group_enabled,
            SenderType::Guild(_) => |v| v.guild_enabled,
        };
        let mut cmd_line = match &parsed_sender {
            SenderType::Console(evt) => evt.line.clone(),
            SenderType::Private(evt) => evt.raw_message.clone(),
            SenderType::Group(evt) => evt.raw_message.clone(),
            SenderType::Guild(evt) => evt.message.to_string(),
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
                    info!(
                        "<{}> issueing command: {}",
                        parsed_sender.generate_sender_message(),
                        cmd_line
                    );
                    match self.command_manager.touch_command_and_test_timeout(
                        cmd.command_name.as_str(),
                        self.config.command_cooldown,
                        &parsed_sender,
                    ) {
                        Ok(v) => {
                            if !v {
                                self.create_client()
                                    .quick_send_by_sender(
                                        &parsed_sender,
                                        format!("指令 {} 正在冷却，请稍等", cmd.command_name)
                                            .as_str(),
                                    )
                                    .await
                                    .ok();
                                return;
                            }
                        }
                        Err(e) => {
                            error!("无法获取指令延时信息:\n{}", e);
                            self.create_client()
                                .quick_send_by_sender(
                                    &parsed_sender,
                                    format!("无法获取指令延时信息:\n{}", e).as_str(),
                                )
                                .await
                                .ok();
                            return;
                        }
                    };
                    if enable_checker(&cmd) {
                        if cmd.plugin_name.as_ref().unwrap() == "<bot>" {
                            let mut args = splitted
                                .iter()
                                .map(|x| String::from(*x))
                                .collect::<Vec<String>>();
                            args.remove(0);
                            let call_result = self
                                .on_command(cmd.command_name.clone(), args, parsed_sender.clone())
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
                            let plugin_wrapper_guard = (self)
                                .plugin_manager
                                .plugins
                                .get(cmd_local.plugin_name.as_ref().unwrap())
                                .unwrap()
                                .read()
                                .await;
                            let plugin = plugin_wrapper_guard.plugin_instance.clone();
                            let cmd_name = cmd.command_name.clone();
                            let mut args = splitted
                                .iter()
                                .map(|x| String::from(*x))
                                .collect::<Vec<String>>();
                            args.remove(0);
                            let sender_cloned = parsed_sender.clone();
                            let client_cloned = self.create_client();
                            let cmd_cloned = cmd.clone();
                            // let should_use_command_handler = plugin_wrapper_guard.use_command_handler;
                            tokio::spawn(async move {
                                let local_sender = sender_cloned;
                                let local_cmd = cmd_cloned;
                                let call_ret = (if let Some(handler) = &local_cmd.command_handler {
                                    trace!("Handling command through handler..");
                                    handler
                                        .lock()
                                        .await
                                        .on_command(cmd_name, args, &local_sender, plugin.clone())
                                        .await
                                } else {
                                    trace!("Handling command through plugin..");
                                    plugin
                                        .write()
                                        .await
                                        .on_command(cmd_name, args, &local_sender)
                                        .await
                                })
                                .map_err(|e| anyhow!(format!("{}", e)));
                                trace!("Command process done.");
                                if let Err(e) = call_ret {
                                    // let err2 = anyhow!(format!("{}", e));
                                    error!("{:#?}", e);
                                    client_cloned
                                        .quick_send_by_sender(
                                            &local_sender,
                                            format!("执行指令时发生错误:\n{}", &e).as_str(),
                                        )
                                        .await
                                        .ok();
                                };
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
                SenderType::Guild(evt) => {
                    client
                        .quick_send(&MessageEvent::Guild(evt.clone()), &s)
                        .await
                        .ok();
                }
            }
        }
    }
}
