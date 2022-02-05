use std::sync::Arc;

use log::info;

use crate::countdown_bot::{
    command::{Command, SenderType},
    plugin::PluginLoadSource,
};

use super::CountdownBot;

impl CountdownBot {
    pub async fn on_command_plugins(
        &mut self,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = String::new();
        for (name, plugin) in self.plugin_manager.plugins.iter() {
            buf.push_str(
                format!(
                    "{}\n来源: {}\n版本: {}\n作者: {}\n介绍: {}\n\n",
                    name,
                    (match plugin.load_source {
                        PluginLoadSource::Static => "静态加载",
                        PluginLoadSource::Dynamic(_) => "动态加载",
                    }),
                    plugin.meta.version,
                    plugin.meta.author,
                    plugin.meta.description,
                )
                .as_str(),
            );
        }
        self.client
            .clone()
            .unwrap()
            .quick_send_by_sender(sender, &buf)
            .await?;
        Ok(())
    }
    pub async fn on_command_about(
        &mut self,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.create_client()
            .quick_send_by_sender(
                &sender,
                &format!(
                    r#"Countdown-Bot 3, version {}
By MikuNotFoundException
https://github.com/Officeyutong/Countdown-Bot3"#,
                    env!("CARGO_PKG_VERSION")
                ),
            )
            .await
            .ok();
        Ok(())
    }
    pub async fn on_command_status(
        &mut self,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let state_str = self
            .state_manager
            .create_state(&self.plugin_manager)
            .await?;
        self.create_client()
            .quick_send_by_sender(&sender, &state_str)
            .await
            .ok();
        Ok(())
    }
    pub async fn on_command_server_status(
        &mut self,
        _sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let val = self.create_client().get_status().await.unwrap();
        info!("{:#?}", val);
        Ok(())
    }
    pub async fn on_command_server_version(
        &mut self,
        _sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let val = self.create_client().get_version_info().await.unwrap();
        info!("{:#?}", val);
        Ok(())
    }
    pub async fn on_command_stop(
        &mut self,
        _sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.stop_signal_sender.as_ref().unwrap().send(true).ok();
        Ok(())
    }
    pub async fn on_command_help(
        &mut self,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
            buf.push_str(format!(" --- {}\n", cmd.description.as_str()).as_str());
        }
        self.create_client()
            .quick_send_by_sender(&sender, &buf)
            .await
            .ok();
        Ok(())
    }
    pub async fn on_command(
        &mut self,
        command: String,
        _args: Vec<String>,
        sender: SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command.as_str() {
            "help" => self.on_command_help(&sender).await,
            "stop" => self.on_command_stop(&sender).await,
            "server_status" => self.on_command_server_status(&sender).await,
            "server_version" => self.on_command_server_version(&sender).await,
            "status" => self.on_command_status(&sender).await,
            "about" => self.on_command_about(&sender).await,
            "plugins" => self.on_command_plugins(&sender).await,
            _ => {
                panic!("?")
            }
        }
    }
}
