use countdown_bot3::countdown_bot::plugin::{self, PluginMeta};
use log::info;
use plugin::PluginRegistrar;
pub struct DemoPlugin;
impl plugin::BotPlugin for DemoPlugin {
    fn on_enable(
        &self,
        bot: &mut countdown_bot3::countdown_bot::bot::CountdownBot,
    ) -> Result<(), Box<(dyn std::error::Error)>> {
        log::set_logger(bot.get_logger())?;
        log::set_max_level(bot.get_max_log_level());
        bot.echo(&String::from("Message from plugin: qaqaq"));
        info!("My log output!");
        return Ok(());
    }

    fn on_disable(
        &self,
        _bot: &mut countdown_bot3::countdown_bot::bot::CountdownBot,
    ) -> Result<(), Box<(dyn std::error::Error)>> {
        return Ok(());
        // todo!()
    }

    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("officeyutong"),
            description: String::from("qaq"),
            version: 1.0,
            name: String::from("demo"),
        }
    }
}
countdown_bot3::export_plugin!(register);
extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_plugin("demo", Box::new(DemoPlugin));
}
