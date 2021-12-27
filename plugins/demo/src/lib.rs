use anyhow::anyhow;
use countdown_bot3::countdown_bot::{
    bot,
    client::CountdownBotClient,
    command::{Command, CommandHandler, SenderType},
    event::{Event, EventContainer},
    plugin::{self, PluginMeta},
};
use log::{debug, info};
static PLUGIN_NAME: &str = "demo";
pub struct DemoPlugin {
    client: Option<CountdownBotClient>,
}
impl DemoPlugin {
    pub fn new() -> Self {
        Self { client: None }
    }
}
pub struct WhoamiHandler {
    client: CountdownBotClient,
}
#[async_trait::async_trait]
impl CommandHandler for WhoamiHandler {
    async fn on_command(
        &mut self,
        _command: String,
        _args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sender_uid = match &sender {
            SenderType::Console(_) => return Err(Box::from(anyhow!("Unexpected sender!"))),
            SenderType::Private(evt) => evt.user_id,
            SenderType::Group(evt) => evt.user_id,
        };
        let info = self
            .client
            .get_stranger_info(sender_uid.into(), false)
            .await
            .unwrap();
        self.client
            .quick_send_by_sender(&sender, &format!("{:?}", info))
            .await
            .ok();
        Ok(())
    }
}
#[async_trait::async_trait]
impl plugin::BotPlugin for DemoPlugin {
    async fn on_schedule_loop(&mut self, name: &str) {
        match name {
            "main_loop" => {
                info!("Loop!");
            }
            _ => {}
        };
    }
    async fn on_state_hook(&mut self) -> String {
        String::new()
    }
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
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.client.clone().unwrap();
        match command.as_str() {
            "test_command" => {
                client
                    .quick_send_by_sender(&sender, &format!("{:?}", args))
                    .await
                    .ok();
                Ok(())
            }

            _ => {
                panic!("?")
            }
        }
    }
    fn on_enable(
        &mut self,
        bot: &mut countdown_bot3::countdown_bot::bot::CountdownBot,
    ) -> Result<(), Box<(dyn std::error::Error)>> {
        countdown_bot3::initialize_plugin_logger!(bot);
        bot.register_command(
            Command::new("test_command")
                .description("qaqqaqqwq")
                .group(true)
                .private(true)
                .console(true),
        )?;

        bot.register_schedule((0, 58), String::from("main_loop"));
        return Ok(());
    }
    fn on_before_start(
        &mut self,
        bot: &mut bot::CountdownBot,
        client: CountdownBotClient,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.client = Some(client);
        bot.register_command(
            Command::new("whoami")
                .description("查询我的信息")
                .group(true)
                .private(true)
                .handler(Box::new(WhoamiHandler {
                    client: self.client.clone().unwrap(),
                })),
        )?;
        Ok(())
    }
    async fn on_disable(&mut self) -> Result<(), Box<(dyn std::error::Error)>> {
        return Ok(());
    }

    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("officeyutong"),
            description: String::from("qaq"),
            version: String::from("1.0"),
        }
    }
}
countdown_bot3::export_plugin!(PLUGIN_NAME, DemoPlugin::new());
