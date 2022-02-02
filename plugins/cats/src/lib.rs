use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use async_trait::async_trait;
use clap::{App, Arg};
use config::CatsConfig;
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
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
static PLUGIN_NAME: &str = "cats";

mod cat_impl;
mod config;
mod misc_impl;
mod models;
pub mod tencent_cloud;
mod upload_impl;
struct CatsPlugin {
    client: Option<CountdownBotClient>,
    config: Option<CatsConfig>,
    database: Option<Arc<Mutex<Connection>>>,
    last_try: Arc<Mutex<HashMap<i64, i64>>>,
}
impl Default for CatsPlugin {
    fn default() -> Self {
        Self {
            client: Default::default(),
            config: Default::default(),
            database: None,
            last_try: Default::default(),
        }
    }
}
#[async_trait]
impl BotPlugin for CatsPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        self.database = Some(Arc::new(Mutex::new(Connection::open(
            bot.ensure_plugin_data_dir(PLUGIN_NAME)?.join("cats.db"),
        )?)));
        bot.register_command(
            Command::new("cat")
                .group(true)
                .private(true)
                .description("吸猫 | 使用 cat --help 查看帮助"),
        )?;
        bot.register_command(
            Command::new("upload")
                .group(true)
                .private(true)
                .description("上传猫片 | upload <图片> | upload --help 查看帮助"),
        )?;

        bot.register_command(
            Command::new("list-cats")
                .enable_all()
                .description("查看上传过猫片的用户列表 | list-cats [用户QQ号]"),
        )?;
        bot.register_command(
            Command::new("delete-cat")
                .enable_all()
                .description("删除猫片 | delete-cat <ID>"),
        )?;
        let cloned = self.database.as_ref().unwrap().clone();
        tokio::spawn(async move {
            init_database(cloned)
                .await
                .expect("初始化数据库时发生错误!");
        });
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
            description: String::from("吸猫插件"),
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
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command.as_str() {
            "delete-cat" => {
                if args.is_empty() {
                    return Err(anyhow!("请输入要删除的ID!").into());
                }
                self.delete_cat(sender, &args[0]).await?;
            }
            "list-cats" => {
                self.list_cat(sender, args.get(0).map(|x| x.clone()))
                    .await?;
            }
            "cat" => {
                let parse_result = App::new("cat")
                    .before_help("查看猫片。\n不指定任何参数的情况下，从数据库中随机；\n指定QQ的情况下，从指定QQ中随机;\n指定ID的情况下，输出指定ID的猫片。")
                    .arg(
                        Arg::with_name("qq")
                            .long("qq")
                            .help("只选择该QQ的猫片")
                            .takes_value(true),
                    )
                    .arg(
                        Arg::with_name("id")
                            .long("id")
                            .help("输出指定ID的猫片")
                            .takes_value(true),
                    )
                    .setting(clap::AppSettings::NoBinaryName)
                    .setting(clap::AppSettings::DisableVersion)
                    .get_matches_from_safe(args);
                match parse_result {
                    Ok(parse_ret) => {
                        debug!("{:#?}", parse_ret);
                        self.cat_command(sender, parse_ret).await?;
                    }
                    Err(e) => {
                        error!("{:#?}", e);
                        self.client
                            .as_ref()
                            .unwrap()
                            .quick_send_by_sender(&sender, e.message.as_str())
                            .await?;
                    }
                };
            }
            "upload" => {
                let parse_result = App::new("upload")
                    .before_help("上传猫片")
                    .arg(
                        Arg::with_name("as-qq")
                            .long("as-qq")
                            .help("以某用户的身份上传")
                            .takes_value(true),
                    )
                    .arg(
                        Arg::with_name("IMAGE")
                            .help("图片数据")
                            .required(true)
                            .index(1),
                    )
                    .setting(clap::AppSettings::NoBinaryName)
                    .setting(clap::AppSettings::DisableVersion)
                    .get_matches_from_safe(args);
                match parse_result {
                    Ok(parse_ret) => {
                        debug!("{:#?}", parse_ret);
                        self.upload_command(sender, parse_ret).await?;
                    }
                    Err(e) => {
                        error!("{:#?}", e);
                        self.client
                            .as_ref()
                            .unwrap()
                            .quick_send_by_sender(&sender, e.message.as_str())
                            .await?;
                    }
                };
            }
            _ => todo!(),
        };
        Ok(())
    }
}

export_static_plugin!(PLUGIN_NAME, CatsPlugin::default());
async fn init_database(conn: Arc<Mutex<Connection>>) -> ResultType<()> {
    let db = conn.lock().await;
    db.execute(
        r#"CREATE TABLE IF NOT EXISTS CATS(
        ID INTEGER PRIMARY KEY AUTOINCREMENT,
        USER_ID INTEGER NOT NULL,
        UPLOAD_ID INTEGER,
        DATA BLOB NOT NULL,
        CHECKSUM TEXT NOT NULL UNIQUE
    )"#,
        params![],
    )?;
    db.execute("CREATE INDEX ID_INDEX ON CATS(ID)", params![])
        .ok();
    db.execute("CREATE INDEX USER_ID_INDEX ON CATS(USER_ID)", params![])
        .ok();
    db.execute("CREATE INDEX INDEX_CHECKSUM ON CATS(CHECKSUM)", params![])
        .ok();

    Ok(())
}
