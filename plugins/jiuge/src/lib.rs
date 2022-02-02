use async_trait::async_trait;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::{CountdownBotClient, ResultType},
        command::{Command, SenderType},
        event::EventContainer,
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    export_static_plugin,
};
use log::{debug, error};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::help_str::HELP_STRING;

static PLUGIN_NAME: &str = "jiuge";
mod help_str;
mod r#impl;
#[derive(Deserialize, Serialize)]
pub struct JiugeConfig {
    pub time_limit: u32,
    pub retry_times: u32,
    pub root_url: String,
}
impl Default for JiugeConfig {
    fn default() -> Self {
        Self {
            retry_times: 50,
            time_limit: 30,
            root_url: "http://jiuge.thunlp.org".to_string(),
        }
    }
}
impl JiugeConfig {
    pub fn sub_url(&self, url: &str) -> ResultType<Url> {
        Ok(Url::parse(self.root_url.as_str())?.join(url)?)
    }
}
#[derive(Deserialize)]
pub struct MostSimpleResp {
    pub code: i32,
}
struct JiugePlugin {
    client: Option<CountdownBotClient>,
    config: Option<JiugeConfig>,
    http_client: reqwest::Client,
}

impl Default for JiugePlugin {
    fn default() -> Self {
        Self {
            client: None,
            config: None,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BotPlugin for JiugePlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        bot.register_command(
            Command::new("jiuge")
                .group(true)
                .private(true)
                .description("九歌爬虫 | 使用指令 jiuge --help 查看帮助"),
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
            description: String::from("九歌爬虫"),
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
        let client = self.client.clone().unwrap();
        let config = self.config.as_ref().unwrap();
        let help_string = HELP_STRING
            .to_string()
            .replace("<PLACEHOLDER>", config.root_url.as_str());
        use clap::{App, Arg};
        let parse_result = App::new("jiuge")
            .help(help_string.as_str())
            .arg(
                Arg::with_name("image")
                    .short("i")
                    .long("image")
                    .help("是否输出图片"),
            )
            .arg(
                Arg::with_name("genre")
                    .short("g")
                    .long("genre")
                    .help("体裁,默认为1,具体取值见帮助")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("yan")
                    .short("y")
                    .long("yan")
                    .help("言数(5或7)，默认为5")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("style")
                    .short("s")
                    .long("style")
                    .help("风格(默认为0，具体见帮助)")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("KEYWORD")
                    .help("诗歌关键词")
                    .required(true)
                    .index(1),
            )
            .setting(clap::AppSettings::ColorNever)
            .setting(clap::AppSettings::NoBinaryName)
            .before_help(HELP_STRING)
            .get_matches_from_safe(args);
        match parse_result {
            Ok(parse_ret) => {
                debug!("{:#?}", parse_ret);
                self.handle_command(sender, &parse_ret).await?;
            }
            Err(parse_err) => {
                error!("{:?}", parse_err);
                client
                    .quick_send_by_sender(&sender, parse_err.message.as_str())
                    .await?;
            }
        };
        Ok(())
    }
}

export_static_plugin!(PLUGIN_NAME, JiugePlugin::default());
