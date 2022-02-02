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

mod r#impl;
static PLUGIN_NAME: &str = "music_163";

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginMode {
    Phone,
    Email,
}

#[derive(Deserialize, Serialize)]
pub struct Music163Config {
    pub api_url: String,
    pub search_limit: u32,
    pub will_login: bool,
    pub login_mode: LoginMode,
    pub phone: String,
    pub email: String,
    pub password: String,
}
impl Default for Music163Config {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:3000".to_string(),
            search_limit: 10,
            will_login: false,
            login_mode: LoginMode::Email,
            phone: "phone_here".to_string(),
            email: "email_here".to_string(),
            password: "password".to_string(),
        }
    }
}
impl Music163Config {
    pub fn sub_url(&self, url: &str) -> ResultType<Url> {
        Ok(Url::parse(self.api_url.as_str())?.join(url)?)
    }
}
#[derive(Deserialize)]
pub struct MostSimpleResp {
    pub code: i32,
}
struct Music163Plugin {
    client: Option<CountdownBotClient>,
    config: Option<Music163Config>,
    http_client: reqwest::Client,
}

impl Default for Music163Plugin {
    fn default() -> Self {
        Self {
            client: None,
            config: None,
            http_client: reqwest::ClientBuilder::new()
                .cookie_store(true)
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl BotPlugin for Music163Plugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        bot.register_command(
            Command::new("music")
                .group(true)
                .private(true)
                .description("网易云音乐搜歌 | 使用指令 music --help 查看帮助"),
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
            author: String::from("Antares"),
            description: String::from("网易云音乐推歌"),
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
        use clap::{App, Arg};
        let parse_result = App::new("music")
            .help_message("显示帮助信息")
            .before_help("网易云音乐搜歌")
            .arg(
                Arg::with_name("KEYWORD")
                    .help("搜索关键词")
                    .required(true)
                    .multiple(true),
            )
            .arg(
                Arg::with_name("id")
                    .short("i")
                    .long("id")
                    .help("使用ID查询歌曲"),
            )
            .arg(
                Arg::with_name("share")
                    .short("s")
                    .long("share")
                    .help("发送分享卡片"),
            )
            .arg(
                Arg::with_name("url")
                    .short("u")
                    .long("url")
                    .help("发送下载链接"),
            )
            .arg(
                Arg::with_name("record")
                    .short("r")
                    .long("record")
                    .help("发送录音(默认)"),
            )
            .setting(clap::AppSettings::ColorNever)
            .setting(clap::AppSettings::NoBinaryName)
            .setting(clap::AppSettings::DisableVersion)
            .get_matches_from_safe(args);
        match parse_result {
            Ok(parse_ret) => {
                debug!("{:#?}", parse_ret);
                self.handle_command(sender, parse_ret).await?;
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

export_static_plugin!(PLUGIN_NAME, Music163Plugin::default());
