use std::time::Duration;

use anyhow::anyhow;
use async_trait::async_trait;
use config::MessageSenderConfig;
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
use log::{debug, error, info};

static PLUGIN_NAME: &str = "message_sender";
mod config;

struct MessageSenderPlugin {
    client: Option<CountdownBotClient>,
    config: Option<MessageSenderConfig>,
}

impl Default for MessageSenderPlugin {
    fn default() -> Self {
        Self {
            client: None,
            config: None,
        }
    }
}

#[async_trait]
impl BotPlugin for MessageSenderPlugin {
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
            Command::new("sendmsg")
                .group(true)
                // .guild(true)
                .private(true)
                .console(true)
                .description("发送消息 | 使用 sendmsg --help 查看帮助"),
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
            description: String::from("消息发送"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_command(
        &mut self,
        _command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cmd_line = args
            .join(" ")
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        use clap::{App, Arg};
        let parse_result = App::new("sendmsg")
            .before_help("快捷发送消息。如果不指定参数，则默认发送到当前对话环境")
            .arg(
                Arg::new("group")
                    .short('g')
                    .long("group")
                    .help("要发送到的群(群号或all,可以多个)")
                    .takes_value(true)
                    .multiple_occurrences(true)
                    .required(false),
            )
            .arg(
                Arg::new("user")
                    .short('u')
                    .long("user")
                    .help("要发送到的用户(可以多个)")
                    .takes_value(true)
                    .multiple_occurrences(true)
                    .required(false),
            )
            .arg(
                Arg::new("message")
                    .help("要发送的消息")
                    .required(true)
                    .multiple_values(true)
                    .index(1),
            )
            .setting(clap::AppSettings::NoBinaryName)
            .setting(clap::AppSettings::DisableVersionFlag)
            .color(clap::ColorChoice::Never)
            .try_get_matches_from(cmd_line);

        match parse_result {
            Ok(parse_ret) => {
                debug!("{:#?}", parse_ret);
                self.handle_sendmsg(parse_ret, sender).await?;
            }
            Err(parse_err) => {
                error!("{:}", parse_err);
                self.client
                    .as_ref()
                    .unwrap()
                    .quick_send_by_sender(&sender, &parse_err.to_string())
                    .await?;
            }
        };
        return Ok(());
    }
}
impl MessageSenderPlugin {
    async fn handle_sendmsg(&self, args: clap::ArgMatches, sender: &SenderType) -> ResultType<()> {
        let config = self.config.as_ref().unwrap();
        let can_use_command = match sender {
            SenderType::Console(_) => true,
            ty @ SenderType::Group(_) | ty @ SenderType::Private(_) => {
                config.whitelist_users.contains(match ty {
                    SenderType::Private(v) => &v.user_id,
                    SenderType::Group(v) => &v.user_id,
                    _ => return Err(anyhow!("非法发送者类型!").into()),
                })
            }
            _ => todo!(),
        };
        if !can_use_command {
            return Err(anyhow!("你没有权限使用此指令!").into());
        }
        let send_groups = args.values_of("group").map(|v| v.collect::<Vec<&str>>());
        let send_users = args.values_of("user").map(|v| v.collect::<Vec<&str>>());
        let message = html_escape::decode_html_entities(
            &args
                .values_of("message")
                .map(|v| v.collect::<Vec<&str>>().join(" "))
                .ok_or(anyhow!("请输入消息!"))?,
        )
        .to_string();
        info!("Sending message: {}", message);
        let client = self.client.as_ref().unwrap();
        if send_groups.is_none() && send_users.is_none() {
            client
                .quick_send_by_sender_ex(sender, &message, false)
                .await?;
        }
        if let Some(users) = send_users {
            for user in users {
                let uid = i64::from_str_radix(user, 10).map_err(|_| anyhow!("非法QQ: {}", user))?;
                info!("Send to {}", user);
                client.send_private_msg(uid, &message, false).await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        if let Some(groups) = send_groups {
            let send_all = groups.iter().any(|s| *s == "all");
            let real_to_send = if send_all {
                let all_groups = client.get_group_list().await?;
                all_groups.into_iter().map(|v| v.group_id).collect()
            } else {
                let mut out = vec![];
                for c in groups.iter() {
                    out.push(i64::from_str_radix(c, 10).map_err(|_| anyhow!("非法群号: {}", c))?);
                }
                out
            };
            for grp in real_to_send.into_iter() {
                info!("Sending to group {}", grp);
                client.send_group_msg(grp, &message, false).await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        return Ok(());
    }
}

export_static_plugin!(PLUGIN_NAME, MessageSenderPlugin::default());
