use anyhow::anyhow;
use async_trait::async_trait;
use chrono::Local;
use config::DockerRunnerConfig;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::CountdownBotClient,
        command::{Command, SenderType},
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    export_static_plugin,
};
use log::debug;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

static PLUGIN_NAME: &str = "docker_runner";

mod config;
mod exec_impl;
mod misc_impl;
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct CacheSourceTuple {
    pub uid: i64,
    pub gid: i64,
}
pub struct CacheEntry {
    pub input_data: String,
    pub inserted_at: chrono::DateTime<Local>,
}
struct DockerRunnerPlugin {
    client: Option<CountdownBotClient>,
    config: Option<DockerRunnerConfig>,
    input_cache: Arc<Mutex<BTreeMap<CacheSourceTuple, CacheEntry>>>,
}

impl Default for DockerRunnerPlugin {
    fn default() -> Self {
        Self {
            client: None,
            config: None,
            input_cache: Default::default(),
        }
    }
}

#[async_trait]
impl BotPlugin for DockerRunnerPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        debug!("Config: {:#?}", self.config);
        bot.register_command(
            Command::new("exec")
                .group(true)
                .description("执行Python代码 | exec <代码>"),
        )?;
        bot.register_command(
            Command::new("execx")
                .group(true)
                .description("执行代码 | execx <语言> <代码>"),
        )?;
        bot.register_command(
            Command::new("input")
                .group(true)
                .description("指定下一次执行程序时的标准输入 | input <数据>"),
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
            description: String::from("在Docker中执行代码"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
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

impl DockerRunnerPlugin {
    pub async fn handle_command(
        &mut self,
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command.as_str() {
            "exec" => {
                let loc_args = args.join(" ");
                let cmd_line = html_escape::decode_html_entities(&loc_args);
                let sender_evt = match sender {
                    SenderType::Group(e) => e,
                    _ => todo!(),
                };
                let real_code = format!(
                    "CALLER_UID={}\n{}\n{}\n{}",
                    sender_evt.user_id,
                    make_assign_str(
                        "CALLER_NICKNAME",
                        &sender_evt
                            .sender
                            .nickname
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or("")
                    ),
                    make_assign_str(
                        "CALLER_CARD",
                        &sender_evt
                            .sender
                            .card
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or("")
                    ),
                    cmd_line
                );
                self.handle_exec(sender, &real_code, "python").await?;
                return Ok(());
            }
            "execx" => {
                if args.is_empty() {
                    self.client
                        .clone()
                        .unwrap()
                        .quick_send_by_sender(
                            sender,
                            format!(
                                "当前支持的语言有:\n{}",
                                self.config
                                    .as_ref()
                                    .unwrap()
                                    .language_setting
                                    .keys()
                                    .map(|x| x.clone())
                                    .collect::<Vec<String>>()
                                    .join(" ")
                            )
                            .as_str(),
                        )
                        .await?;
                    return Ok(());
                }
                if args.len() == 1 {
                    return Err(anyhow!("请输入代码!").into());
                }
                let lang_id = &args[0];
                let code_joined = args[1..].join(" ");
                let code = html_escape::decode_html_entities(&code_joined);
                self.handle_exec(sender, &code, &lang_id).await?;
                return Ok(());
            }
            "input" => {
                let items = args.join(" ");
                self.handle_input(sender, &items).await?;
                return Ok(());
            }
            _ => todo!(),
        };
    }
}

export_static_plugin!(PLUGIN_NAME, DockerRunnerPlugin::default());

fn make_assign_str(var: &str, val: &str) -> String {
    let b64enc = base64::encode(val.as_bytes());
    return format!(
        "{}=__import__('base64').decodebytes(b\"{}\").decode()",
        var, b64enc
    );
}
