use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{Duration, TimeZone};
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::{CountdownBotClient, ResultType},
        command::{Command, SenderType},
        event::EventContainer,
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    export_static_plugin, initialize_plugin_logger,
};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
static PLUGIN_NAME: &str = "broadcast";
#[derive(Deserialize, Serialize, Clone)]
pub struct BroadcastEntry {
    pub name: String,
    // Y-M-D
    pub date: String,
}
impl BroadcastEntry {
    pub fn parse_date(&self) -> ResultType<(i32, u32, u32)> {
        let parsed = self.date.split("-").collect::<Vec<&str>>();
        if parsed.len() != 3 {
            return Err(anyhow!("日期格式错误: {}", self.date).into());
        }
        return Ok((
            i32::from_str_radix(parsed[0], 10)?,
            u32::from_str_radix(parsed[1], 10)?,
            u32::from_str_radix(parsed[2], 10)?,
        ));
    }
}
type BroadcastMap = HashMap<String, Vec<BroadcastEntry>>;
#[derive(Deserialize, Serialize, Clone)]
pub struct BroadcastPluginConfig {
    pub broadcast_hour: u32,
    pub broadcast_minute: u32,
    pub using_url_list: bool,
    pub list_url: String,
    /*
        #     "群号": [
    #         {
    #             "name": "广播名",
    #             "date": "年-月-日"
    #         }
    #     ]
    */
    pub list: BroadcastMap,
}
impl Default for BroadcastPluginConfig {
    fn default() -> Self {
        Self {
            broadcast_hour: 6u32,
            broadcast_minute: 0,
            using_url_list: false,
            list_url: "".to_string(),
            list: HashMap::new(),
        }
    }
}
#[derive(Default)]
struct BroadcastPlugin {
    client: Option<CountdownBotClient>,
    config: Option<BroadcastPluginConfig>,
}

#[async_trait]
impl BotPlugin for BroadcastPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        initialize_plugin_logger!(bot);
        bot.register_command(
            Command::new("broadcast")
                .description("在当前群进行广播")
                .group(true)
                .single_alias("广播"),
        )?;
        bot.register_state_hook();
        self.config = Some(load_config_or_save_default::<BroadcastPluginConfig>(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        let cfg = self.config.as_ref().unwrap();
        bot.register_schedule(
            (cfg.broadcast_hour, cfg.broadcast_minute),
            String::from("倒计时广播"),
        );
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
            description: String::from("群广播"),
            version: String::from("2.0"),
        }
    }
    async fn on_event(&mut self, _event: EventContainer) -> HookResult<()> {
        Ok(())
    }

    async fn on_state_hook(&mut self) -> HookResult<String> {
        let config = self.config.as_ref().unwrap();
        return Ok(format!(
            "倒计时广播时间: 每天{:0>2}:{:0>2}",
            config.broadcast_hour, config.broadcast_minute
        ));
    }
    async fn on_schedule_loop(&mut self, _name: &str) -> HookResult<()> {
        let broadcast_data = self.ensure_broadcast_data().await?;
        for (group, data) in broadcast_data.iter() {
            self.broadcast_at_group(group.as_str(), data).await?;
        }
        Ok(())
    }

    async fn on_command(
        &mut self,
        _command: String,
        _args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let SenderType::Group(evt) = sender {
            let group_id = evt.group_id;
            let broadcast_data = self.ensure_broadcast_data().await.unwrap();
            if let Some((group_str, data)) = broadcast_data.get_key_value(&group_id.to_string()) {
                self.broadcast_at_group(group_str, data).await.unwrap();
            } else {
                return Err(anyhow!("当前群无广播数据!").into());
            }
            return Ok(());
        } else {
            panic!("?");
        }
    }
}
impl BroadcastPlugin {
    pub async fn ensure_broadcast_data(&self) -> ResultType<BroadcastMap> {
        let config = self.config.as_ref().unwrap();
        if config.using_url_list {
            let resp = serde_json::from_str::<BroadcastMap>(
                reqwest::get(config.list_url.clone())
                    .await?
                    .text()
                    .await?
                    .as_str(),
            )?;
            return Ok(resp);
        } else {
            return Ok(config.list.clone());
        }
    }
    async fn broadcast_at_group(
        &self,
        group: &str,
        broadcasts: &Vec<BroadcastEntry>,
    ) -> ResultType<()> {
        let client = self.client.clone().unwrap();
        let ret = generate_broadcast_content(broadcasts)?;

        for item in ret.iter() {
            client
                .send_group_msg(i64::from_str_radix(group, 10)?, item, false)
                .await?;
        }
        Ok(())
    }
}

fn generate_broadcast_content(broadcast_list: &Vec<BroadcastEntry>) -> ResultType<Vec<String>> {
    let mut result = Vec::<String>::new();
    let today = chrono::Local::today();
    for item in broadcast_list.iter() {
        let name = &item.name;
        let (y, m, d) = item.parse_date()?;
        let countdown_date = chrono::prelude::Local.ymd(y, m, d);
        let diff = countdown_date - today + Duration::days(1);
        let months = diff.num_days() / 30;
        let days = diff.num_days() % 30;
        if diff.num_days() < 0 {
            continue;
        }
        let text: String = if diff.num_days() > 0 {
            if months > 0 {
                format!(
                    "距离 {} 还有 {} 天 ({} 个月 {})",
                    name,
                    diff.num_days(),
                    months,
                    (if days != 0 {
                        format!("{}天", days)
                    } else {
                        "整".to_string()
                    })
                )
            } else {
                format!("距离 {} 还有 {} 天", name, diff.num_days())
            }
        } else {
            format!("今天是 {} ", name)
        };
        debug!("Generated text: {}", &text);
        result.push(text);
    }
    return Ok(result);
}

export_static_plugin!(PLUGIN_NAME, BroadcastPlugin::default());
