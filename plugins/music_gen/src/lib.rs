use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use config::MusicGenConfig;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::CountdownBotClient,
        command::{Command, SenderType},
        event::EventContainer,
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    export_static_plugin,
};
use tokio::sync::Semaphore;
static PLUGIN_NAME: &str = "music_gen";

mod cache;
mod command;
mod command_entry;
mod config;
mod help;
pub mod luogu_fetcher;
pub mod notes;
mod pysynth;
mod utils;
struct MusicGenPlugin {
    client: Option<CountdownBotClient>,
    config: Option<MusicGenConfig>,
    semaphore: Option<Arc<tokio::sync::Semaphore>>,
    redis_client: Option<Arc<redis::Client>>,
}
impl Default for MusicGenPlugin {
    fn default() -> Self {
        Self {
            client: Default::default(),
            config: Default::default(),
            semaphore: None,
            redis_client: None,
        }
    }
}
#[async_trait]
impl BotPlugin for MusicGenPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(
            load_config_or_save_default(&bot.ensure_plugin_data_dir(PLUGIN_NAME)?)
                .map_err(|e| anyhow!("加载配置时发生错误: {}\n{}", e, e.backtrace()))?,
        );
        self.semaphore = Some(Arc::new(Semaphore::new(
            self.config.as_ref().unwrap().max_execute_sametime as usize,
        )));
        self.redis_client = Some(Arc::new(redis::Client::open(
            self.config.as_ref().unwrap().redis_uri.as_str(),
        )?));
        bot.register_command(
            Command::new("musicgen")
                .group(true)
                .description("生成音乐 | 使用 musicgen --help 查看帮助"),
        )
        .unwrap();

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
    async fn on_disable(&mut self) -> HookResult<()> {
        Ok(())
    }
    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("officeyutong"),
            description: String::from("音乐生成"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_event(&mut self, _event: EventContainer) -> HookResult<()> {
        Ok(())
    }

    async fn on_state_hook(&mut self) -> HookResult<String> {
        Ok(String::new())
    }
    async fn on_schedule_loop(&mut self, _name: &str) -> HookResult<()> {
        Ok(())
    }

    async fn on_command(
        &mut self,
        _command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.command_entry(args, sender, true).await?;
        return Ok(());
    }
}

export_static_plugin!(PLUGIN_NAME, MusicGenPlugin::default());
