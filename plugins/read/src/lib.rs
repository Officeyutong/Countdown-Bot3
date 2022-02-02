use anyhow::anyhow;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::{CountdownBotClient, ResultType},
        command::{Command, SenderType},
        event::EventContainer,
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    initialize_plugin_logger,
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;

static PLUGIN_NAME: &str = "read";
#[derive(Deserialize, Serialize, Debug)]
struct ReadConfig {
    pub max_string_length: u64,
    pub app_id: String,
    pub api_key: String,
    pub secret_key: String,
    pub volume: i64,
    pub speed: i64,
}
impl Default for ReadConfig {
    fn default() -> Self {
        Self {
            max_string_length: 300,
            volume: 8,
            speed: 4,
            api_key: "".to_string(),
            app_id: "".to_string(),
            secret_key: "".to_string(),
        }
    }
}
// #[derive(Default)]
pub struct ReadPlugin {
    client: Option<CountdownBotClient>,
    config: Option<ReadConfig>,
    http_client: reqwest::Client,
}
impl Default for ReadPlugin {
    fn default() -> Self {
        Self {
            client: Default::default(),
            config: Default::default(),
            http_client: reqwest::Client::new(),
        }
    }
}
#[async_trait::async_trait()]
impl BotPlugin for ReadPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        initialize_plugin_logger!(bot);
        bot.register_command(
            Command::new("read")
                .description("文字转语音 | read <文本>")
                .group(true)
                .private(true),
        )?;
        self.config = Some(load_config_or_save_default::<ReadConfig>(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
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
            description: String::from("文字转语音"),
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
        self.handle_command(sender, &args.join(" ")).await
    }
}

countdown_bot3::export_static_plugin!(PLUGIN_NAME, ReadPlugin::default());

impl ReadPlugin {
    async fn get_token(&self) -> ResultType<String> {
        let config = self.config.as_ref().unwrap();
        #[derive(Deserialize)]
        struct Resp {
            pub access_token: String,
        }
        let text = self
            .http_client
            .post("https://openapi.baidu.com/oauth/2.0/token")
            .query(&[
                ("grant_type", "client_credentials"),
                ("client_id", config.api_key.as_str()),
                ("client_secret", config.secret_key.as_str()),
            ])
            .send()
            .await
            .map_err(|e| anyhow!("发送HTTP请求时发生错误: {}", e))?
            .text()
            .await
            .map_err(|e| anyhow!("读取回复时发生错误: {}", e))?;
        info!("JSON Resp: {}", text);
        let json_resp = serde_json::from_str::<Resp>(&text)
            .map_err(|e| anyhow!("反序列化时发生错误: {}", e))?;

        return Ok(json_resp.access_token);
    }
    async fn get_voice(&self, text: &str, token: &str) -> ResultType<Vec<u8>> {
        let config = self.config.as_ref().unwrap();
        let resp = self
            .http_client
            .post("https://tsn.baidu.com/text2audio")
            .form(&[
                ("tex", urlencoding::encode(text).to_string().as_str()),
                ("tok", token),
                ("cuid", "qwqqwqqwq"),
                ("ctp", "1"),
                ("spd", config.speed.to_string().as_str()),
                ("per", "4"),
                ("vol", config.volume.to_string().as_str()),
                ("lan", "zh"),
            ])
            .send()
            .await
            .map_err(|e| anyhow!("发送HTTP请求时发生错误: {}", e))?;
        let ret_bytes: Vec<u8> = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("读取回复时发生错误: {}", e))?
            .to_vec();
        return Ok(ret_bytes);
    }
    async fn handle_command(&self, sender: &SenderType, text: &str) -> ResultType<()> {
        let config = self.config.as_ref().unwrap();
        if text.chars().count() > config.max_string_length as usize {
            return Err(anyhow!("字符串过长！最长为 {} 字符。", config.max_string_length).into());
        }
        let token = self
            .get_token()
            .await
            .map_err(|e| anyhow!("获取Token时发生错误，请检查相关设置：\n{}", e))?;
        let record_bytes = self.get_voice(text, &token).await?;
        // let json = serde_json::<Value>::from_str(re)
        {
            let local_ret: ResultType<Value> = (|| {
                let s = String::from_utf8(record_bytes.clone())?;
                let v = serde_json::from_str::<Value>(&s)?;
                return Ok(v);
            })();
            if let Ok(v) = local_ret {
                return Err(anyhow!("生成语音时发生错误: {}", v).into());
            }
        }
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender(
                sender,
                &format!("[CQ:record,file=base64://{}]", base64::encode(record_bytes)),
            )
            .await?;
        return Ok(());
    }
}
