use anyhow::anyhow;
use async_trait::async_trait;
use bullshit::generate_bullshit;
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
// use serde::{Deserialize, Serialize};
static PLUGIN_NAME: &str = "bullshit";
mod bullshit;
mod bullshit_data;
mod shit;
#[derive(Default)]
struct BullshitPlugin {
    client: Option<CountdownBotClient>,
}

#[async_trait]
impl BotPlugin for BullshitPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        bot.register_command(
            Command::new("bullshit")
                .description("生成狗屁不通文章 | bullshit <主题>")
                .enable_all(),
        )
        .unwrap();
        bot.register_command(
            Command::new("shit")
                .description("将输入字符串分词后随机打乱 | shit <字符串>")
                .enable_all(),
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
            description: String::from("狗屁不通文章生成器"),
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
        return self.handle_command(command, args, sender).await;
    }
}
impl BullshitPlugin {
    pub async fn handle_command(
        &self,
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command.as_str() {
            "bullshit" => {
                if args.is_empty() {
                    return Err(anyhow!("请输入主题!").into());
                }
                let args_joined = args.join(" ");
                self.client
                    .clone()
                    .unwrap()
                    .quick_send_by_sender(sender, generate_bullshit(&args_joined).as_str())
                    .await?;
            }
            "shit" => {
                if args.is_empty() {
                    return Err(anyhow!("请输入主题!").into());
                }
                let args_joined = args.join(" ");
                let mut vec_chars = args_joined.chars().collect::<Vec<char>>();
                if vec_chars.len() > 500 {
                    vec_chars = vec_chars.iter().take(500).map(|x| *x).collect();
                }
                let real_args = vec_chars.iter().collect::<String>();
                self.command_shit(&real_args, sender).await?;
            }
            _ => todo!("?"),
        };
        Ok(())
    }
}
export_static_plugin!(PLUGIN_NAME, BullshitPlugin::default());
