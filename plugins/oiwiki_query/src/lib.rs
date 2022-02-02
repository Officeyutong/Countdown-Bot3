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
static PLUGIN_NAME: &str = "oiwiki_query";

#[derive(Default)]
struct OIWikiQueryPlugin {
    client: Option<CountdownBotClient>,
}

#[derive(Deserialize)]
struct QueryRespEntry {
    pub url: String,
    pub title: String,
}

#[async_trait]
impl BotPlugin for OIWikiQueryPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        bot.register_command(
            Command::new("oiwiki")
                .enable_all()
                .description("OI-Wiki查询 | oiwiki <查询关键字>"),
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
            description: String::from("OI-Wiki查询"),
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
        if let Some(_) = args.get(0) {
            let resp = serde_json::from_str::<Vec<QueryRespEntry>>(
                reqwest::Client::new()
                    .get("https://search.oi-wiki.org:8443")
                    .query(&[("s", args.join(" ").as_str())])
                    .send()
                    .await?
                    .text()
                    .await?
                    .as_str(),
            )?;
            let mut buf = format!("查询到 {} 条相关内容:\n", resp.len());
            for item in resp.iter() {
                buf.push_str(
                    format!("{}: https://oi-wiki.org{}\n\n", item.title, item.url).as_str(),
                );
            }
            self.client
                .clone()
                .unwrap()
                .quick_send_by_sender(&sender, buf.as_str())
                .await?;
            return Ok(());
        } else {
            return Err(anyhow!("请输入查询关键字!").into());
        }
    }
}

export_static_plugin!(PLUGIN_NAME, OIWikiQueryPlugin::default());
