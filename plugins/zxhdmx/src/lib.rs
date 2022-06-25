use std::{
    any::TypeId,
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use async_trait::async_trait;
use config::ZxhdmxConfig;
use countdown_bot3::{
    countdown_bot::{
        bot::{self, CountdownBot},
        client::{CountdownBotClient, ResultType},
        command::{Command, SenderType},
        event::{
            manager::{EventListener, WrappedOOPEventContainer},
            message::GroupMessageEvent,
        },
        plugin::{BotPlugin, BotPluginWrapped, HookResult, PluginMeta},
        utils::{load_config_or_save_default, SubUrlWrapper},
    },
    export_static_plugin,
};
use handle_impl::{command::handle_command, event::handle_event};
use log::{debug, info};
use pytypes::{
    wrapped_bot::{MyPyLogger, WrappedCountdownBot},
    wrapped_plugin::{WrappedConfig, WrappedPlugin},
};
use pyvm::{
    builtins::{PyBaseExceptionRef, PyModule},
    PyObjectRef, PyRef, PySettings, PyClassImpl,
};
use rustpython_vm as pyvm;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::web::{GetDataHandler, SetDataHandler, TemplateGetter};

static PLUGIN_NAME: &str = "zxhdmx";
pub mod config;
mod handle_impl;
mod help_str;
pub mod pytypes;
pub mod utils;
mod web;
pub type DataType = Arc<RwLock<Value>>;
pub type GameObjectType = Arc<Mutex<HashMap<i64, PyObjectRef>>>;
pub type InprType = Arc<Mutex<pyvm::Interpreter>>;
pub type HTMLTemplateType = Arc<Mutex<String>>;
struct ZxhdmxPlugin {
    client: Option<CountdownBotClient>,
    config: Option<ZxhdmxConfig>,
    py_inpr: InprType,
    data_dir: Option<PathBuf>,
    game_data: DataType,
    game_module: Option<PyRef<PyModule>>,
    game_objects: GameObjectType,
    url_wrapper: Option<SubUrlWrapper>,
    html_template: HTMLTemplateType,
}
impl Default for ZxhdmxPlugin {
    fn default() -> Self {
        Self {
            client: Default::default(),
            config: Default::default(),
            py_inpr: Arc::new(Mutex::new(pyvm::Interpreter::new_with_init(
                PySettings::default(),
                |vm| {
                    vm.add_native_modules(rustpython_stdlib::get_module_inits());
                    return pyvm::InitParameter::External;
                },
            ))),
            data_dir: None,
            game_data: Arc::new(RwLock::new(Value::Null)),
            game_module: None,
            game_objects: Arc::new(Mutex::new(HashMap::new())),
            url_wrapper: None,
            html_template: Arc::new(Mutex::new(String::new())),
        }
    }
}
#[async_trait]
impl BotPlugin for ZxhdmxPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.data_dir = Some(bot.ensure_plugin_data_dir(PLUGIN_NAME)?);
        self.config = Some(
            load_config_or_save_default(self.data_dir.as_ref().unwrap())
                .map_err(|e| anyhow!("加载配置时发生错误: {}\n{}", e, e.backtrace()))?,
        );
        self.py_inpr.lock().unwrap().enter(|vm| {
            MyPyLogger::make_class(&vm.ctx);
            WrappedCountdownBot::make_class(&vm.ctx);
            WrappedConfig::make_class(&vm.ctx);
            WrappedPlugin::make_class(&vm.ctx);
        });
        self.url_wrapper = Some(bot.create_url_wrapper());
        init_files(self.data_dir.as_ref().unwrap()).expect("Failed to init files!");
        bot.register_command(
            Command::new("zxhdmx")
                .group(true)
                .description("进入真心话大冒险模式"),
        )
        .unwrap();
        bot.register_command(
            Command::new("zxhdmx-reload")
                .console(true)
                .description("重新加载zxhdmx的游戏内容数据"),
        )
        .unwrap();
        bot.register_event_handler(TypeId::of::<GroupMessageEvent>(), MyEventHandler {});
        self.setup_salvo(bot);
        Ok(())
    }
    fn on_before_start(
        &mut self,
        _bot: &mut bot::CountdownBot,
        client: CountdownBotClient,
    ) -> HookResult<()> {
        self.client = Some(client);
        let game_py_str = std::fs::read_to_string(self.data_dir.as_ref().unwrap().join("game.py"))
            .expect("Failed to read 'game.py'");
        self.game_module = Some(
            self.py_inpr
                .lock()
                .unwrap()
                .enter(|vm| {
                    let code_obj = vm
                        .compile(
                            &game_py_str,
                            pyvm::compile::Mode::Exec,
                            "game.py".to_string(),
                        )
                        .map_err(|e| vm.new_syntax_error(&e))?;
                    let module =
                        pyvm::import::import_codeobj(vm, "game", code_obj.code.clone(), true)?
                            .downcast::<PyModule>()
                            .unwrap();
                    return Ok(module);
                })
                .map_err(|e: PyBaseExceptionRef| {
                    anyhow!(
                        "Failed to compile game.py:\nArgs: {:#?}\nTraceback: {:#?}",
                        e.args().as_slice(),
                        e.traceback()
                    )
                })
                .unwrap(),
        );
        info!("Game module: {:#?}", self.game_module.as_ref().unwrap());
        self.reload_gamedata()?;
        self.reload_template()?;
        return Ok(());
    }
    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("officeyutong"),
            description: String::from("真心话大冒险"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_command(
        &mut self,
        command: String,
        _args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if command.as_str() == "zxhdmx" {
            let (user_id, group_id) = match sender {
                SenderType::Group(evt) => (evt.user_id, evt.group_id),
                _ => todo!(),
            };
            if !self
                .config
                .as_ref()
                .unwrap()
                .enable_groups
                .contains(&group_id)
            {
                return Err(anyhow!("当前群未启用本功能!").into());
            }
            let local_client = self.client.clone().unwrap();
            let local_game_data = self.game_data.clone();
            let local_game_module = self.game_module.clone().unwrap();
            let local_game_objects = self.game_objects.clone();
            let local_inpr = self.py_inpr.clone();
            let local_config = self.config.clone().unwrap();
            debug!("Before entering..");
            tokio::task::spawn_blocking(move || {
                return handle_command(
                    user_id,
                    group_id,
                    local_client,
                    local_game_data,
                    local_game_module,
                    local_game_objects,
                    local_inpr,
                    local_config,
                )
                .map_err(|e| anyhow!("{}", e));
            })
            .await??;
        } else if command.as_str() == "zxhdmx-reload" {
            self.reload_gamedata()?;
            self.reload_template()?;
        } else {
            panic!("Invalid command: {}", command);
        }

        // self.handle_command(user_id, group_id).await?;
        return Ok(());
    }
}
impl ZxhdmxPlugin {
    fn reload_template(&self) -> ResultType<()> {
        let file_path = self.data_dir.as_ref().unwrap().join("edit.html");
        *self.html_template.lock().unwrap() = if !file_path.exists() {
            let html_str = include_str!("edit.html");
            std::fs::write(file_path, html_str)
                .map_err(|e| anyhow!("写出默认的 edit.html 时发生错误!\n{}", e))?;
            html_str.to_string()
        } else {
            std::fs::read_to_string(file_path)
                .map_err(|e| anyhow!("读取 edit.html 时发生错误!\n{}", e))?
        };

        return Ok(());
    }
    fn reload_gamedata(&self) -> ResultType<()> {
        {
            let _guard = tokio::runtime::Handle::current().enter();
            let mut game_data = futures::executor::block_on(self.game_data.write());
            *game_data = serde_json::from_str::<Value>(
                std::fs::read_to_string(self.data_dir.as_ref().unwrap().join("data.json"))
                    .map_err(|e| anyhow!("Failed to read data.json:\n{}", e))
                    .unwrap()
                    .as_str(),
            )
            .map_err(|e| anyhow!("Failed to parse data.json:\n{}", e))
            .unwrap();
            info!("Game data loaded.");
            debug!("Game data:\n{:#?}", game_data);
        }
        return Ok(());
    }
    fn setup_salvo(&self, bot: &mut CountdownBot) {
        use salvo::prelude::*;
        let mut router = Router::with_path("zxhdmx");
        router
            .routers_mut()
            .push(Router::with_path("set_data").post(SetDataHandler {
                data: self.game_data.clone(),
                config: self.config.clone().unwrap(),
                data_dir: self.data_dir.clone().unwrap(),
            }));
        router
            .routers_mut()
            .push(Router::with_path("get_data").post(GetDataHandler {
                data: self.game_data.clone(),
                config: self.config.clone().unwrap(),
            }));
        router
            .routers_mut()
            .push(Router::with_path("edit").get(TemplateGetter {
                data: self.html_template.clone(),
            }));
        bot.get_salvo_router().routers_mut().push(router);
    }
}
export_static_plugin!(PLUGIN_NAME, ZxhdmxPlugin::default());
fn init_files(dir_path: &PathBuf) -> ResultType<()> {
    {
        let game_py = include_bytes!("game.py");
        let game_py_path = dir_path.join("game.py");
        if !game_py_path.exists() {
            std::fs::write(game_py_path, game_py)?;
        }
    }
    {
        let default_data = include_bytes!("data.json");
        let data_json_path = dir_path.join("data.json");
        if !data_json_path.exists() {
            std::fs::write(data_json_path, default_data)?;
        }
    }
    return Ok(());
}

struct MyEventHandler;
#[async_trait]
impl EventListener for MyEventHandler {
    async fn on_event(
        &mut self,
        event: WrappedOOPEventContainer,
        plugin: BotPluginWrapped,
    ) -> ResultType<()> {
        debug!("Entered event handler..");
        let plugin_guard = plugin.read().await;
        let casted = plugin_guard.downcast_ref::<ZxhdmxPlugin>().unwrap();
        let event_guard = event.read().await.event.clone();
        let gevt = event_guard.downcast_ref::<GroupMessageEvent>().unwrap();
        let (user_id, group_id, message) = (gevt.user_id, gevt.group_id, gevt.raw_message.clone());
        let pyobj = if let Some(game_obj) = casted.game_objects.lock().unwrap().get(&group_id) {
            game_obj.clone()
        } else {
            return Ok(());
        };
        let inpr = casted.py_inpr.clone();
        let local_client = casted.client.clone().unwrap();
        tokio::task::spawn_blocking(move || {
            handle_event(user_id, group_id, inpr, pyobj, message, local_client)
                .map_err(|e| anyhow!("{}", e))
        })
        .await??;
        return Ok(());
    }
}
