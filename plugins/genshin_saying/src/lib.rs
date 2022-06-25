use anyhow::anyhow;
use async_trait::async_trait;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::{CountdownBotClient, ResultType},
        command::{Command, SenderType},
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    export_static_plugin,
};
use jieba_rs::Jieba;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
static PLUGIN_NAME: &str = "genshin_saying";

#[derive(Deserialize, Serialize)]
pub struct GenshinSayingConfig {
    pub xamaran_template: String,
}

impl Default for GenshinSayingConfig {
    fn default() -> Self {
        Self {
            xamaran_template: "今日{xx}，明日{xx}，百日之后{yy}中已不剩{yy}，只剩{xx}。".into(),
        }
    }
}

#[derive(Default)]
struct GenshinSayingPlugin {
    client: Option<CountdownBotClient>,
    config: Option<GenshinSayingConfig>,
    jieba: Arc<Jieba>,
}

#[async_trait]
impl BotPlugin for GenshinSayingPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default::<GenshinSayingConfig>(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        bot.register_command(
            Command::new("xamaran")
                .description("生成赞玛兰风格语录: xamaran <xx> <yy>")
                .enable_all(),
        )?;
        bot.register_command(
            Command::new("neko")
                .description("生成寝子风格语录(对我、你等文字进行替换): neko <文本内容>")
                .enable_all(),
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
            description: String::from("赞玛兰风格语录与寝子风格语录"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_command(
        &mut self,
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command.as_str() {
            "xamaran" => self.xamaran(&args, sender).await?,
            "neko" => self.neko(&args, sender).await?,
            _ => {}
        };
        Ok(())
    }
}

impl GenshinSayingPlugin {
    async fn xamaran(&self, args: &Vec<String>, sender: &SenderType) -> ResultType<()> {
        let client = self.client.as_ref().unwrap();
        if let [x, y] = &args[..] {
            let cfg_str = self.config.as_ref().unwrap().xamaran_template.clone();
            let mut hmap = HashMap::<String, String>::new();
            hmap.insert("xx".into(), x.into());
            hmap.insert("yy".into(), y.into());

            let result = strfmt::strfmt(&cfg_str, &hmap).map_err(|e| {
                error!("非法格式控制字符串 {}: {}", e, cfg_str);
                anyhow!("非法格式控制字符串!")
            })?;
            client.quick_send_by_sender(sender, &result).await?;
            return Ok(());
        } else {
            return Err(anyhow!("需要两个参数！").into());
        }
    }
    async fn neko(&self, args: &Vec<String>, sender: &SenderType) -> ResultType<()> {
        let text = args.join(" ");
        // let splitted =
        // let jieba = Jieba::new();
        debug!("Nekonize input: {:?}", text);
        let jieba = self.jieba.clone();
        let result: anyhow::Result<String> = tokio::task::spawn_blocking(move || {
            let words = jieba.cut(&text, false);
            debug!("Splitted words: {:?}", words);
            let result = words
                .iter()
                .map(|s| match *s {
                    "我" => "奴家".into(),
                    "你" => "汝等".into(),
                    "。" => "，喵喵。".into(),
                    t => t.to_string(),
                })
                .collect::<Vec<String>>()
                .join("");
            // words
            Ok(result)
        })
        .await?;
        let client = self.client.as_ref().unwrap();

        client.quick_send_by_sender(sender, &result?).await?;
        return Ok(());
    }
}

export_static_plugin!(PLUGIN_NAME, GenshinSayingPlugin::default());
