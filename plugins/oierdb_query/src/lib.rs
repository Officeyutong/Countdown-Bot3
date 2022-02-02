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
use rand::prelude::SliceRandom;
use serde::Deserialize;
static PLUGIN_NAME: &str = "oierdb_query";

#[derive(Default)]
struct OIerDBQueryPlugin {
    client: Option<CountdownBotClient>,
}
#[derive(Deserialize)]
pub struct QueryRespEntry {
    pub awards: String,
    pub id: String,
    pub level: String,
    pub name: String,
    pub pinyin: String,
    pub rel1: Option<String>,
    pub rel2: Option<String>,
    pub score: String,
    pub sex: String,
    pub smth: String,
    pub year: String,
}
#[derive(Deserialize)]
pub struct AwardEntry {
    pub award_type: String,
    pub ctype: String,
    pub grade: String,
    pub identity: String,
    pub province: String,
    pub rank: i32,
    pub school: String,
    pub school_id: u64,
    pub score: String,
}
#[async_trait]
impl BotPlugin for OIerDBQueryPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        bot.register_command(
            Command::new("oier")
                .enable_all()
                .description("OIerDB(bytew.net/OIer) 查询 | oier <关键词>"),
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
            description: String::from("OIerDB(http://bytew.net/OIer)查询"),
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
        if let Some(keyword) = args.get(0) {
            let mut buf = String::from("查询到以下数据:\n");
            #[derive(Deserialize)]
            struct Resp {
                pub result: Vec<QueryRespEntry>,
            }
            let resp = reqwest::Client::new()
                .get("https://bytew.net/OIer/search.php")
                .query(&[("method", "normal"), ("q", keyword.as_str())])
                .send()
                .await?;
            let mut parsed = serde_json::from_str::<Resp>(resp.text().await?.as_str())?.result;
            parsed.shuffle(&mut rand::thread_rng());

            for entry in parsed[0..std::cmp::min(parsed.len(), 5)].iter() {
                let sex = match entry.sex.as_str() {
                    "1" => "男",
                    "-1" => "女",
                    _ => "未知",
                };
                buf.push_str(format!("姓名: {}\n生理性别: {}\n", entry.name, sex).as_str());
                let awards = serde_json::from_str::<Vec<AwardEntry>>(
                    entry.awards.replace("'", "\"").as_str(),
                )?;
                for award in awards.iter() {
                    buf.push_str(format!(
                        "在<{province}>{school}<{grade}>时参加<{contest}>以{score}分(全国排名{rank})的成绩获得<{type}>\n",
                        province=award.province,
                        school=award.school,
                        grade=award.grade,
                        contest=award.identity,
                        score=award.score,
                        rank=award.rank,
                        r#type=award.award_type,    
                    ).as_str());
                }
                buf.push_str("\n");
            }
            if parsed.len() > 5 {
                buf.push_str("请去原网站查看完整数据!");
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

export_static_plugin!(PLUGIN_NAME, OIerDBQueryPlugin::default());
