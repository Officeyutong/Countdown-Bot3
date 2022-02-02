use anyhow::anyhow;
use async_trait::async_trait;
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
use serde::{Deserialize, Serialize};
mod r#impl;
pub mod sam;
static PLUGIN_NAME: &str = "ds_drawer_plugin";
#[derive(Deserialize, Serialize)]
pub struct DSDrawerConfig {
    pub max_string_length: u32,
    pub dot_executable: String,
    pub dot_timeout: i32,
}
impl Default for DSDrawerConfig {
    fn default() -> Self {
        Self {
            max_string_length: 20,
            dot_executable: String::from("dot"),
            dot_timeout: 30,
        }
    }
}

struct DSDrawerPlugin {
    client: Option<CountdownBotClient>,
    config: Option<DSDrawerConfig>,
}

impl Default for DSDrawerPlugin {
    fn default() -> Self {
        Self {
            client: None,
            config: None,
        }
    }
}

#[async_trait]
impl BotPlugin for DSDrawerPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        bot.register_command(
            Command::new("sam")
                .group(true)
                .private(true)
                .description("绘制后缀自动机 | sam <字符串(使用|分割不同的字符串)>"),
        )?;
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
            description: String::from("SAM绘制器"),
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
        if args.is_empty() {
            return Err(anyhow!("请输入字符串!").into());
        }
        let s = args.join(" ");
        self.generate_sam(&s, sender).await?;
        Ok(())
    }
}

export_static_plugin!(PLUGIN_NAME, DSDrawerPlugin::default());
