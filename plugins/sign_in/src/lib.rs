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
use rusqlite::{params, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;
static PLUGIN_NAME: &str = "sign_in";
use serde::{Deserialize, Serialize};

mod command_group_query_impl;
mod command_sign_in_impl;
mod command_user_query_impl;
mod misc_impl;
mod models;

#[derive(Deserialize, Serialize)]
pub struct SignInConfig {
    pub black_list_groups: Vec<i64>,
    pub hide_score_groups: Vec<i64>,
}

impl Default for SignInConfig {
    fn default() -> Self {
        Self {
            black_list_groups: vec![888888888],
            hide_score_groups: vec![888888888],
        }
    }
}
struct SignInPlugin {
    client: Option<CountdownBotClient>,
    config: Option<SignInConfig>,
    database: Option<Arc<Mutex<Connection>>>,
}
impl Default for SignInPlugin {
    fn default() -> Self {
        Self {
            client: Default::default(),
            config: Default::default(),
            database: None,
        }
    }
}
#[async_trait]
impl BotPlugin for SignInPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(
            load_config_or_save_default(&bot.ensure_plugin_data_dir(PLUGIN_NAME)?)
                .map_err(|e| anyhow!("加载配置时发生错误: {}\n{}", e, e.backtrace()))?,
        );
        self.database = Some(Arc::new(Mutex::new(
            Connection::open(bot.ensure_plugin_data_dir(PLUGIN_NAME)?.join("sign_in.db"))
                .map_err(|e| anyhow!("加载数据库时发生错误: {}", e))?,
        )));
        bot.register_command(
            Command::new("sign-in")
                .group(true)
                .description("签到")
                .single_alias("签到")
                .single_alias("check-in"),
        )
        .unwrap();
        bot.register_command(
            Command::new("签到积分")
                .private(true)
                .description("签到积分查询"),
        )
        .unwrap();
        bot.register_command(
            Command::new("签到记录")
                .group(true)
                .description("签到记录查询 | 签到记录 [月份(可选)] [年份(可选)]"),
        )
        .unwrap();
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
    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("Antares"),
            description: String::from("群签到"),
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
            "sign-in" => {
                self.command_signin(sender).await?;
            }
            "签到积分" => {
                self.command_user_query(sender).await?;
            }
            "签到记录" => {
                self.command_group_query(&args, sender).await?;
            }
            _ => todo!(),
        };
        return Ok(());
    }
}

export_static_plugin!(PLUGIN_NAME, SignInPlugin::default());
async fn init_database(conn: Arc<Mutex<Connection>>) -> ResultType<()> {
    let db = conn.lock().await;
    db.execute(
        r#"CREATE TABLE IF NOT EXISTS SIGNINS(
            GROUP_ID      INTEGER NOT NULL,
            USER_ID       INTEGER NOT NULL,
            TIME          INTEGER NOT NULL,
            DURATION      INTEGER NOT NULL,
            SCORE         INTEGER NOT NULL,
            SCORE_CHANGES INTEGER NOT NULL
        )"#,
        params![],
    )
    .map_err(|e| anyhow!("创建表 SIGNINS 时发生错误: {}", e))?;
    db.execute(
        r#"CREATE TABLE IF NOT EXISTS USERS(
            GROUP_ID INTEGER NOT NULL,
            USER_ID  INTEGER NOT NULL,
            SCORE    INTEGER NOT NULL
        )"#,
        params![],
    )
    .map_err(|e| anyhow!("创建表 USERS 时发生错误: {}", e))?;
    db.execute(
        "CREATE INDEX SIGNIN_GROUP_ID_INDEX ON SIGNINS(GROUP_ID)",
        params![],
    )
    .ok();
    db.execute(
        "CREATE INDEX SIGNIN_USER_ID_INDEX  ON SIGNINS(USER_ID)",
        params![],
    )
    .ok();
    db.execute(
        "CREATE INDEX SIGNIN_TIME_INDEX     ON SIGNINS(TIME)",
        params![],
    )
    .ok();
    db.execute(
        "CREATE INDEX USERS_GROUP_ID_INDEX  ON USERS(GROUP_ID)",
        params![],
    )
    .ok();
    db.execute(
        "CREATE INDEX USERS_USER_ID_INDEX   ON USERS(USER_ID)",
        params![],
    )
    .ok();

    Ok(())
}
