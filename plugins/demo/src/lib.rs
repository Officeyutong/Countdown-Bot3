
use std::{sync::Arc, time::Duration};

use countdown_bot3::countdown_bot::{
    client::CountdownBotClient,
    command::{Command, SenderType},
    event::{Event, EventContainer},
    plugin::{self, PluginMeta},
};
use log::debug;
use plugin::PluginRegistrar;
use tokio::sync::Mutex;
static PLUGIN_NAME: &str = "demo";
pub struct DemoPlugin {}

#[async_trait::async_trait]
impl plugin::BotPlugin for DemoPlugin {
    async fn on_event(&mut self, event: EventContainer) -> bool {
        if let Event::Message(evt) = event.event {
            debug!("Message! {:?}", evt);
        }
        return true;
    }
    async fn on_command(
        &mut self,
        command: String,
        args: Vec<String>,
        sender: SenderType,
        client: CountdownBotClient,
    ) {
        
        match command.as_str() {
            "test_command" => {
                client
                    .quick_send_by_sender(&sender, &format!("{:?}", args))
                    .await
                    .ok();
            }
            _ => {}
        };
    }
    fn on_enable(
        &mut self,
        bot: &mut countdown_bot3::countdown_bot::bot::CountdownBot,
    ) -> Result<(), Box<(dyn std::error::Error)>> {
        log::set_logger(bot.get_logger())?;
        log::set_max_level(bot.get_max_log_level());
        bot.echo(&String::from("Message from plugin: qaqaq"));
        bot.register_command(
            Command::new("test_command")
                .description("qaqqaqqwq")
                .group(true)
                .private(true)
                .console(true),
        )?;
        return Ok(());
    }

    async fn on_disable(
        &mut self,
        _client: CountdownBotClient
    ) -> Result<(), Box<(dyn std::error::Error)>> {
        tokio::time::sleep(Duration::from_secs(4)).await;
        return Ok(());
    }

    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("officeyutong"),
            description: String::from("qaq"),
            version: 1.0,
        }
    }
}
countdown_bot3::export_plugin!(register, PLUGIN_NAME);
#[allow(improper_ctypes_definitions)]
extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_plugin(Arc::new(Mutex::new(DemoPlugin{})));
}
