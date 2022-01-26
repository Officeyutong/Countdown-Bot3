use std::sync::Arc;

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
use log::error;
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
static PLUGIN_NAME: &str = "sign_in";
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Default)]
pub struct SignInConfig {
    pub black_list_groups: Vec<i64>,
    pub hide_score_groups: Vec<i64>,
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
        self.config = Some(load_config_or_save_default(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        self.database = Some(Arc::new(Mutex::new(Connection::open(
            bot.ensure_plugin_data_dir(PLUGIN_NAME)?.join("sign_in.db"),
        )?)));
        bot.register_command(
            Command::new("sign-in")
                .group(true)
                .description("签到")
                .single_alias("签到")
                .single_alias("check-in"),
        )?;
        bot.register_command(
            Command::new("签到积分")
                .private(true)
                .description("签到积分查询"),
        )?;
        bot.register_command(
            Command::new("签到记录")
                .group(true)
                .description("签到记录查询 | 签到记录 [月份(可选)] [年份(可选)]"),
        )?;
        let cloned = self.database.as_ref().unwrap().clone();
        tokio::spawn(async move {
            if let Err(e) = init_database(cloned).await {
                error!("初始化数据库时发生错误!\n:{}", e);
            }
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
            author: String::from("Antares"),
            description: String::from("群签到"),
            version: String::from("2.0"),
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
            _ => todo!(),
        };
        return Ok(());
    }
}

export_static_plugin!(PLUGIN_NAME, SignInPlugin::default());
async fn init_database(conn: Arc<Mutex<Connection>>) -> ResultType<()> {
    let db = conn.lock().await;
    db.execute(
        r#"""CREATE TABLE IF NOT EXISTS SIGNINS(
            GROUP_ID      INTEGER NOT NULL,
            USER_ID       INTEGER NOT NULL,
            TIME          INTEGER NOT NULL,
            DURATION      INTEGER NOT NULL,
            SCORE         INTEGER NOT NULL,
            SCORE_CHANGES INTEGER NOT NULL
        )"""#,
        params![],
    )?;
    db.execute(
        r#"""CREATE TABLE IF NOT EXISTS USERS(
            GROUP_ID INTEGER NOT NULL,
            USER_ID  INTEGER NOT NULL,
            SCORE    INTEGER NOT NULL
        )"""#,
        params![],
    )?;
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
