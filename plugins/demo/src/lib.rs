#[allow(improper_ctypes_definitions)]

use std::sync::Arc;

use countdown_bot3::countdown_bot::{
    command::{Command, SenderType},
    event::{Event, EventContainer},
    plugin::{self, PluginMeta},
};
use log::{debug, info};
use plugin::PluginRegistrar;
use tokio::sync::Mutex;
static PLUGIN_NAME: &str = "demo";
pub struct DemoPlugin;

#[async_trait::async_trait]
impl plugin::BotPlugin for DemoPlugin {
    async fn on_event(&mut self, event: EventContainer) -> bool {
        if let Event::Message(evt) = event.event {
            debug!("Message! {:?}", evt);
        }
        return true;
    }
    async fn on_command(&mut self, _command: String, _args: Vec<String>, _sender: SenderType) {}
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
                .console(false),
        )?;
        // return Err(Box::new(anyhow!("qaqqaq")));
        info!("Command registered!");
        return Ok(());
    }

    fn on_disable(
        &mut self,
        _bot: &mut countdown_bot3::countdown_bot::bot::CountdownBot,
    ) -> Result<(), Box<(dyn std::error::Error)>> {
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
extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_plugin(Arc::new(Mutex::new(DemoPlugin)));
}
