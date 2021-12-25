use std::sync::Arc;

use log::{debug, info};

use crate::countdown_bot::command::{Command, SenderType};

use super::CountdownBot;

impl CountdownBot {
    pub async fn on_command_server_status(&mut self, _sender: &SenderType) {
        let val = self.create_client().get_status().await.unwrap();
        info!("{:#?}", val);
    }
    pub async fn on_command_server_version(&mut self, _sender: &SenderType) {
        let val = self.create_client().get_version_info().await.unwrap();
        info!("{:#?}", val);
    }
    
    pub async fn on_command_test(&mut self, _sender: &SenderType) {
        let resp = self
            .create_client()
            .send_private_msg(814980678, &String::from("qaqqaq"), true)
            .await;
        debug!("{:?}", resp);
    }
    pub async fn on_command_stop(&mut self, _sender: &SenderType) {
        self.stop_signal_sender.as_ref().unwrap().send(true).ok();
    }
    pub async fn on_command_help(&mut self, sender: &SenderType) {
        let mut buf = String::from("指令列表:\n");
        let cmds = self
            .command_manager
            .command_map
            .iter()
            .filter(match sender {
                SenderType::Console(_) => |p: &(&String, &Arc<Command>)| p.1.console_enabled,
                SenderType::Private(_) => |p: &(&String, &Arc<Command>)| p.1.private_enabled,
                SenderType::Group(_) => |p: &(&String, &Arc<Command>)| p.1.group_enabled,
            })
            .collect::<Vec<(&String, &Arc<Command>)>>();

        for (name, cmd) in cmds.iter() {
            buf.push_str(format!("{}", name).as_str());
            if !cmd.alias.is_empty() {
                buf.push_str(format!("[{}]", cmd.alias.join(",").as_str()).as_str());
            }
            buf.push_str(format!(": {}\n", cmd.description.as_str()).as_str());
        }
        self.create_client()
            .quick_send_by_sender(&sender, &buf)
            .await
            .ok();
    }
    pub async fn on_command(&mut self, command: String, _args: Vec<String>, sender: SenderType) {
        match command.as_str() {
            "help" => self.on_command_help(&sender).await,
            "stop" => self.on_command_stop(&sender).await,
            "test" => self.on_command_test(&sender).await,
            "server_status" => self.on_command_server_status(&sender).await,
            "server_version" => self.on_command_server_version(&sender).await,
            _ => {}
        };
    }
}
