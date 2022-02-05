use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use config::MusicGenConfig;
use countdown_bot3::{
    countdown_bot::{
        bot::{self, CountdownBot},
        client::{CountdownBotClient, ResultType},
        command::{Command, SenderType},
        event::EventContainer,
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::{load_config_or_save_default, SubUrlWrapper},
    },
    export_static_plugin,
};
use reqwest::{header::HeaderValue, StatusCode};
use salvo::{prelude::FlowCtrl, Depot, Handler, Request, Response};
use tokio::sync::Semaphore;
static PLUGIN_NAME: &str = "music_gen";

mod cache;
mod command;
mod command_entry;
mod config;
mod help;
pub mod luogu_fetcher;
pub mod notes;
mod pysynth;
mod utils;
struct MusicGenPlugin {
    client: Option<CountdownBotClient>,
    config: Option<MusicGenConfig>,
    semaphore: Option<Arc<tokio::sync::Semaphore>>,
    redis_client: Option<Arc<redis::Client>>,
    url_wrapper: Option<SubUrlWrapper>,
}
impl Default for MusicGenPlugin {
    fn default() -> Self {
        Self {
            client: Default::default(),
            config: Default::default(),
            semaphore: None,
            redis_client: None,
            url_wrapper: None,
        }
    }
}
#[async_trait]
impl BotPlugin for MusicGenPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(
            load_config_or_save_default(&bot.ensure_plugin_data_dir(PLUGIN_NAME)?)
                .map_err(|e| anyhow!("加载配置时发生错误: {}\n{}", e, e.backtrace()))?,
        );
        self.semaphore = Some(Arc::new(Semaphore::new(
            self.config.as_ref().unwrap().max_execute_sametime as usize,
        )));
        self.redis_client = Some(Arc::new(redis::Client::open(
            self.config.as_ref().unwrap().redis_uri.as_str(),
        )?));
        bot.register_command(
            Command::new("musicgen")
                .group(true)
                .description("生成音乐 | 使用 musicgen --help 查看帮助"),
        )
        .unwrap();
        self.url_wrapper = Some(bot.create_url_wrapper());
        setup_salvo(bot, self.redis_client.as_ref().unwrap().clone());
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
        // if let Some(v) = &self.join_handle {
        //     v.abort();
        // }
        Ok(())
    }
    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("officeyutong"),
            description: String::from("音乐生成"),
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
        self.command_entry(args, sender, true).await?;
        return Ok(());
    }
}

export_static_plugin!(PLUGIN_NAME, MusicGenPlugin::default());

fn setup_salvo(bot: &mut CountdownBot, redis_client: Arc<redis::Client>) {
    use salvo::prelude::*;
    let router =
        Router::with_path("/music_gen/download/<hash>").get(SimpleHandler { redis_client });
    bot.get_salvo_router().routers_mut().push(router);
}

struct SimpleHandler {
    redis_client: Arc<redis::Client>,
}
impl SimpleHandler {
    pub(crate) async fn get_data(&self, hash: &str) -> ResultType<Vec<u8>> {
        let mut conn = self.redis_client.clone().get_async_connection().await?;
        use redis::AsyncCommands;
        let key = cache::make_key(hash);
        if !conn.exists(&key).await? {
            return Err(anyhow!("Invalid hash!").into());
        }
        let resp: Vec<u8> = conn.get(&key).await?;
        return Ok(resp);
    }
}
#[async_trait::async_trait]
impl Handler for SimpleHandler {
    async fn handle(
        &self,
        req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        _ctrl: &mut FlowCtrl,
    ) {
        if let Some(hash) = req.get_param::<String>("hash") {
            match self.get_data(&hash).await {
                Ok(v) => {
                    res.headers_mut().append(
                        "Content-Disposition",
                        HeaderValue::from_str(&format!("attachment; filename={}.wav", hash))
                            .unwrap(),
                    );
                    res.render_binary(HeaderValue::from_static("audio/wave"), &v[..])
                }
                Err(e) => {
                    res.set_status_code(StatusCode::NOT_FOUND);
                    res.render_plain_text(&format!("Error: {}", e));
                }
            };
        } else {
            res.set_status_code(StatusCode::BAD_REQUEST);
            res.render_plain_text("Param required");
        }
    }
}
