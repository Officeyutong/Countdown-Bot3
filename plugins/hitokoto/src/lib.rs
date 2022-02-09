use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::{CountdownBotClient, ResultType},
        command::{Command, SenderType},
        plugin::{BotPlugin, BotPluginWrapped, HookResult, PluginMeta},
        schedule_loop::handler::ScheduleLoopHandler,
        utils::load_config_or_save_default,
    },
    export_static_plugin,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
static PLUGIN_NAME: &str = "hitokoto";

#[derive(Deserialize, Serialize)]
pub struct HitokotoConfig {
    pub broadcast_hour: u32,
    pub broadcast_minute: u32,
    pub using_url_list: bool,
    pub list_url: String,
    pub list_local: Vec<String>,
}

impl Default for HitokotoConfig {
    fn default() -> Self {
        Self {
            broadcast_hour: 6,
            broadcast_minute: 0,
            using_url_list: false,
            list_url: "".to_string(),
            list_local: vec![],
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct Hitokoto {
    pub hitokoto: String,
    pub from: String,
    pub id: u64,
}
impl Hitokoto {
    pub fn generate_message(&self) -> String {
        format!(
            r#"{text}
            
--- {source}
            
(Hitokoto ID:{id} https://hitokoto.cn/?id={id})"#,
            text = self.hitokoto,
            source = self.from,
            id = self.id
        )
    }
}
#[derive(Default)]
struct HitokotoPlugin {
    client: Option<CountdownBotClient>,
    config: Option<HitokotoConfig>,
}
async fn random_hitokoto() -> ResultType<Hitokoto> {
    Ok(serde_json::from_str::<Hitokoto>(
        reqwest::get("https://v1.hitokoto.cn/")
            .await?
            .text()
            .await?
            .as_str(),
    )?)
}
async fn fetch_hitokoto_by_id(id: u32) -> ResultType<Hitokoto> {
    let text = reqwest::get(format!("https://hitokoto.cn?id={}", id))
        .await?
        .text()
        .await?;
    use scraper::{ElementRef, Html, Selector};
    // use soup::prelude::*;
    // let soup = Soup::new(text.as_str());
    let doc = Html::parse_document(text.as_str());
    let text = {
        let text = doc
            .select(&Selector::parse("#hitokoto_text").unwrap())
            .collect::<Vec<ElementRef>>();
        if text.is_empty() {
            return Err(anyhow!("页面爬取错误: 缺失#hitokoto_text元素").into());
        }
        let elem = text[0];
        elem.text().collect::<Vec<&str>>().join("\n")
    };
    let source = {
        let text = doc
            .select(&Selector::parse("#hitokoto_author").unwrap())
            .collect::<Vec<ElementRef>>();
        if text.is_empty() {
            return Err(anyhow!("页面爬取错误: 缺失#hitokoto_author元素").into());
        }
        let elem = text[0];
        elem.text().collect::<Vec<&str>>().join("\n")
    };
    return Ok(Hitokoto {
        from: source,
        hitokoto: text,
        id: id as u64,
    });
}
#[async_trait]
impl BotPlugin for HitokotoPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default::<HitokotoConfig>(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        let cfg = self.config.as_ref().unwrap();
        bot.register_state_hook();
        bot.register_schedule(
            (cfg.broadcast_hour, cfg.broadcast_minute),
            "Hitokoto定时广播".to_string(),
            Arc::new(Mutex::new(ScheduleHandler {})),
        );
        bot.register_command(
            Command::new("hitokoto")
                .enable_all()
                .description("hitokoto - 随机 | hitokoto <ID> - 查询指定ID"),
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
    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("officeyutong"),
            description: String::from("一言广播 & 查询"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_state_hook(&mut self) -> HookResult<String> {
        let cfg = self.config.as_ref().unwrap();
        Ok(format!(
            "Hitokoto广播时间: 每天{:0>2}:{:0>2}",
            cfg.broadcast_hour, cfg.broadcast_minute
        ))
    }

    async fn on_command(
        &mut self,
        _command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let to_send = if let Some(id) = args.get(0) {
            fetch_hitokoto_by_id(u32::from_str_radix(id, 10)?).await?
        } else {
            random_hitokoto().await?
        };
        self.client
            .clone()
            .unwrap()
            .quick_send_by_sender(sender, to_send.generate_message().as_str())
            .await?;
        Ok(())
    }
}

export_static_plugin!(PLUGIN_NAME, HitokotoPlugin::default());

struct ScheduleHandler;
#[async_trait::async_trait]
impl ScheduleLoopHandler for ScheduleHandler {
    async fn on_schedule_loop(&mut self, _name: &str, plugin: BotPluginWrapped) -> HookResult<()> {
        let guard = plugin.read().await;
        let casted_guard = guard.downcast_ref::<HitokotoPlugin>().unwrap();
        let cfg = casted_guard.config.as_ref().unwrap();
        let groups = if cfg.using_url_list {
            serde_json::from_str::<Vec<String>>(
                reqwest::get(cfg.list_url.clone())
                    .await?
                    .text()
                    .await?
                    .as_str(),
            )?
        } else {
            cfg.list_local.clone()
        };

        for group in groups.iter() {
            let client = casted_guard.client.as_ref().unwrap();
            let gid = i64::from_str_radix(group, 10)?;
            info!("Sending hitokoto to group {}", gid);
            let r = random_hitokoto().await?;
            info!("Value: {:#?}", r);
            if let Err(e) = client
                .clone()
                .send_group_msg(gid, r.generate_message().as_str(), false)
                .await
            {
                error!("发送Hitokoto到群 {} 失败:\n{}", gid, e);
            }
        }
        return Ok(());
    }
}
