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

static PLUGIN_NAME: &str = "covid19";
mod common;
mod covid19_impl;
mod covnews_impl;
#[derive(Default)]
struct COVID19Plugin {
    client: Option<CountdownBotClient>,
}
#[async_trait]
impl BotPlugin for COVID19Plugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        bot.register_command(
            Command::new("covid19")
                .single_alias("covid")
                .enable_all()
                .description("查询国内新冠疫情 | covid19 [省份]"),
        )?;
        bot.register_command(
            Command::new("covnews")
                .enable_all()
                .description("查询国内新冠最近五条新闻(丁香医生新闻源)"),
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
            description: String::from("丁香园COVID19数据查询"),
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
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let joined = args.join(" ");
        match command.as_str() {
            "covnews" => self.handle_covnews(sender).await?,
            "covid19" => {
                self.handle_covid19(
                    if args.is_empty() {
                        None
                    } else {
                        Some(joined.as_str())
                    },
                    sender,
                )
                .await?;
            }
            _ => todo!(),
        };
        Ok(())
    }
}

export_static_plugin!(PLUGIN_NAME, COVID19Plugin::default());
