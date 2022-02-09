pub mod command_impl;
use std::path::PathBuf;

use countdown_bot3::{
    countdown_bot::{
        bot,
        client::CountdownBotClient,
        command::{Command, SenderType},
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    initialize_plugin_logger,
};
use serde::{Deserialize, Serialize};

static PLUGIN_NAME: &str = "weather";
#[derive(Deserialize, Serialize, Debug)]
struct WeatherConfig {
    pub api_key: String,
}
impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            api_key: "和风天气API_KEY".to_string(),
        }
    }
}
// #[derive(Default)]
pub struct WeatherPlugin {
    client: Option<CountdownBotClient>,
    plugin_data_root: Option<PathBuf>,
    config: Option<WeatherConfig>,
    // runtime: tokio::runtime::Runtime,
}
impl Default for WeatherPlugin {
    fn default() -> Self {
        Self {
            client: Default::default(),
            plugin_data_root: Default::default(),
            config: Default::default(),
        }
    }
}
#[async_trait::async_trait()]
impl BotPlugin for WeatherPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        initialize_plugin_logger!(bot);
        self.plugin_data_root = Some(bot.ensure_plugin_data_dir(PLUGIN_NAME)?);
        bot.register_command(
            Command::new("weather")
                .description("查询天气 | weather <地名/城市代码/IP地址/经度,纬度> (单个地名半角逗号分割小到大的行政区排列)")
                .console(true)
                .group(true)
                .private(true)
                .single_alias("天气"),
        )?;
        self.config = Some(load_config_or_save_default::<WeatherConfig>(
            &self.plugin_data_root.as_ref().unwrap(),
        )?);
        Ok(())
    }
    fn on_before_start(
        &mut self,
        _bot: &mut bot::CountdownBot,
        client: CountdownBotClient,
    ) -> HookResult<()> {
        self.client = Some(client);
        Ok(())
    }
    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("Antares"),
            description: String::from("天气查询"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_command(
        &mut self,
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.on_weather_command(command, args, sender).await
    }
}

countdown_bot3::export_static_plugin!(PLUGIN_NAME, WeatherPlugin::default());
