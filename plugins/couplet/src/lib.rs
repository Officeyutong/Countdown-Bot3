use anyhow::anyhow;
use async_trait::async_trait;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::CountdownBotClient,
        command::{Command, SenderType},
        event::EventContainer,
        plugin::{BotPlugin, HookResult, PluginMeta},
    },
    export_static_plugin,
};
use serde::Deserialize;
static PLUGIN_NAME: &str = "couplet";

#[derive(Default)]
struct CoupletPlugin {
    client: Option<CountdownBotClient>,
}

#[async_trait]
impl BotPlugin for CoupletPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        bot.register_command(
            Command::new("couplet")
                .description("对联机 | couplet <上联>")
                .enable_all()
                .single_alias("对联"),
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
            author: String::from("Antares"),
            description: String::from("对联机"),
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
        _couplcommand: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if args.is_empty() {
            return Err(anyhow!("请输入上联!").into());
        }
        let keyword = args[0..].join(" ");
        #[derive(Deserialize)]
        struct Resp {
            output: String,
        }
        let resp = reqwest::get(format!(
            "https://ai-backend.binwang.me/chat/couplet/{}",
            keyword
        ))
        .await?;
        let output = serde_json::from_str::<Resp>(resp.text().await?.as_str())?;
        self.client
            .clone()
            .unwrap()
            .quick_send_by_sender(
                sender,
                format!("上联: {}\n下联: {}", keyword, output.output).as_str(),
            )
            .await?;
        Ok(())
    }
}

export_static_plugin!(PLUGIN_NAME, CoupletPlugin::default());
