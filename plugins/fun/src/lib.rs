use async_trait::async_trait;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::{CountdownBotClient, ResultType},
        command::{Command, SenderType},
        event::{message::MessageEvent, Event, EventContainer},
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    export_static_plugin,
};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
static PLUGIN_NAME: &str = "fun";

#[derive(Deserialize, Serialize)]
pub struct FunConfig {
    pub enable_repeater: bool,
    pub blacklist_groups: Vec<i64>,
    pub repeat_time_limit: i32,
    pub repeat_delay: i32,
}

impl Default for FunConfig {
    fn default() -> Self {
        Self {
            blacklist_groups: vec![],
            enable_repeater: true,
            repeat_delay: 3 * 60,
            repeat_time_limit: 3,
        }
    }
}

struct RepeatData {
    pub last_message: Option<String>,
    pub repeat_times: i32,
    pub last_repeat_time: Option<chrono::DateTime<chrono::Local>>,
}

#[derive(Default)]
struct FunPlugin {
    client: Option<CountdownBotClient>,
    config: Option<FunConfig>,
    repeat_data: HashMap<i64, RepeatData>,
}

#[async_trait]
impl BotPlugin for FunPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default::<FunConfig>(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        bot.register_command(Command::new("阿克").description("阿克").enable_all())?;
        bot.register_command(Command::new("爆零").description("qwq").enable_all())?;
        bot.register_command(Command::new("凉了").description("凉了？").enable_all())?;
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
            description: String::from("包括复读机以及一些有意思的小指令"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_event(&mut self, event: EventContainer) -> HookResult<()> {
        let config = self.config.as_ref().unwrap();
        let client = self.client.clone().unwrap();
        match event.event {
            Event::Message(mevt) => match mevt {
                MessageEvent::Group(grpevt) => {
                    let gid = grpevt.group_id;
                    if !config.blacklist_groups.contains(&gid) {
                        handle_repeat(&mut self.repeat_data, gid, client, &grpevt.message, config)
                            .await?;
                    }
                }
                _ => {}
            },
            _ => {}
        };
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
        _args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.client.clone().unwrap();
        match command.as_str() {
            "阿克" => {
                client.quick_send_by_sender(sender, "您阿克了！").await?;
            }
            "爆零" => {
                client
                    .quick_send_by_sender(sender, "您不会爆零的qwq")
                    .await?;
            }
            "凉了" => {
                client
                    .quick_send_by_sender(sender, "qwq您不会凉的~")
                    .await?;
            }
            _ => {}
        };
        Ok(())
    }
}

export_static_plugin!(PLUGIN_NAME, FunPlugin::default());

async fn handle_repeat(
    data: &mut HashMap<i64, RepeatData>,
    gid: i64,
    client: CountdownBotClient,
    msg: &String,
    cfg: &FunConfig,
) -> ResultType<()> {
    if !data.contains_key(&gid) {
        data.insert(
            gid,
            RepeatData {
                last_message: Some(msg.clone()),
                last_repeat_time: None,
                repeat_times: 1,
            },
        );
        return Ok(());
    }
    let mut repdata = data.get_mut(&gid).unwrap();
    if repdata.last_message == Some(msg.clone()) {
        repdata.repeat_times += 1
    } else {
        repdata.repeat_times = 1;
        repdata.last_message = Some(msg.clone());
    }
    if repdata.repeat_times >= cfg.repeat_time_limit {
        let can_repeat = {
            if let Some(last_repeat) = repdata.last_repeat_time {
                let time_diff = chrono::Local::now() - last_repeat;
                if time_diff.num_seconds() < cfg.repeat_delay.into() {
                    false
                } else {
                    true
                }
            } else {
                true
            }
        };
        if can_repeat {
            debug!("Repeating at group {}", gid);
            repdata.last_message = None;
            repdata.repeat_times = 0;
            repdata.last_repeat_time = Some(chrono::Local::now());
            client.send_group_msg(gid, msg, false).await?;
        } else {
            debug!("Ignoring repeat request: group {}", gid);
        }
    }
    Ok(())
}
