use std::collections::HashSet;
use std::path;
use std::time::Duration;
pub type ReceiverMap = std::collections::HashMap<String, SingleCallSender>;
use super::client::{CountdownBotClient, SingleCallSender};
use super::command::{Command, CommandManager};
use super::config::CountdownBotConfig;
use super::plugin::PluginManager;
use super::schedule_loop::ScheduleLoopManager;
use super::state_hook::StateHookManager;
use config::Config;
use futures_util::stream::{SplitSink, SplitStream};
use log::{debug, error, info};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
pub type WriteStreamType =
    Option<SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>;
pub type ReadStreamType =
    Option<SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>;
pub type StopSignalReceiverType = tokio::sync::watch::Receiver<bool>;
pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");
pub static PRESERVED_PLUGIN_NAMES: [&str; 1] = ["<bot>"];
pub struct CountdownBot {
    sys_root: path::PathBuf,
    config: CountdownBotConfig,
    plugin_manager: PluginManager,
    logger_handle: Option<flexi_logger::LoggerHandle>,
    logger: Option<&'static dyn log::Log>,
    max_log_level: Option<log::LevelFilter>,
    stop: bool,
    write_stream: WriteStreamType,
    read_stream: ReadStreamType,
    // receiver_map: ReceiverMap,
    client: Option<CountdownBotClient>,
    stop_signal_sender: Option<tokio::sync::watch::Sender<bool>>,
    stop_signal_receiver: Option<StopSignalReceiverType>,
    command_manager: CommandManager,
    preserved_plugin_names: HashSet<String>,
    state_manager: StateHookManager,
    schedule_loop_manager: ScheduleLoopManager,
}
mod builtin_command_impl;
mod dispatch_impl;
mod load_plugins_impl;
mod start_impl;
impl CountdownBot {
    pub fn create_client(&self) -> CountdownBotClient {
        return self.client.as_ref().unwrap().clone();
    }
    pub fn register_schedule(&mut self, time: (u32, u32), name: String) {
        self.schedule_loop_manager.register(time, name);
    }
    pub fn register_command(&mut self, cmd: Command) -> Result<(), Box<(dyn std::error::Error)>> {
        return self.command_manager.register_command(cmd);
    }
    pub fn get_command_manager(&mut self) -> &mut CommandManager {
        return &mut self.command_manager;
    }
    pub fn get_logger(&self) -> &'static dyn log::Log {
        return self.logger.unwrap();
    }
    pub fn get_max_log_level(&self) -> log::LevelFilter {
        return self.max_log_level.unwrap().clone();
    }
    pub fn echo(&self, s: &String) {
        info!("Echo: {}", s);
    }
    // pub fn get_plugin_ref(&self, name: &String) -> BotPluginWrapped {
    //     let s = self.plugin_manager.plugins.get(name);
    //     return s.unwrap().blocking_lock().plugin.clone();
    // }
    pub fn register_state_hook(&mut self) {
        self.state_manager.register_state_hook();
    }
    pub fn new(sys_root: &path::PathBuf) -> CountdownBot {
        CountdownBot {
            sys_root: sys_root.clone(),
            config: CountdownBotConfig::default(),
            plugin_manager: PluginManager::new(),
            logger_handle: None,
            logger: None,
            max_log_level: None,
            stop: false,
            write_stream: None,
            read_stream: None,
            client: None,
            stop_signal_sender: None,
            stop_signal_receiver: None,
            command_manager: CommandManager::new(),
            preserved_plugin_names: HashSet::from(PRESERVED_PLUGIN_NAMES.map(|x| String::from(x))),
            state_manager: StateHookManager::default(),
            schedule_loop_manager: ScheduleLoopManager::new(),
        }
    }
    pub async fn init(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if !std::path::Path::new("config.json").exists() {
            tokio::fs::write(
                "config.json",
                serde_json::to_string_pretty(&CountdownBotConfig::default())?.as_bytes(),
            )
            .await?;
            return Err(Box::from(anyhow::anyhow!("已创建默认配置文件，请进行修改")));
        }
        let mut cfg = Config::new();
        cfg.merge(config::Config::try_from(&CountdownBotConfig::default())?)?;
        cfg.merge(config::File::with_name("config"))?;
        self.config = cfg.try_into().expect("Cannot deserialize config file!");
        use flexi_logger::{opt_format, Duplicate, FileSpec, Logger, LoggerHandle};
        self.logger_handle = Some::<LoggerHandle>(
            {
                match self.config.debug {
                    true => Logger::try_with_str("debug"),
                    false => Logger::try_with_str("info"),
                }
            }?
            .format(opt_format)
            .log_to_file(
                FileSpec::default()
                    .directory("logs")
                    .basename("countdown_bot"),
            )
            .duplicate_to_stdout(Duplicate::All)
            .start()?,
        );
        self.logger = Some(log::logger());
        self.max_log_level = Some(log::max_level());
        info!("Initializing Countdown-Bot3 ...");
        info!("Currently working path: {}", self.sys_root.display());
        info!("Executable: {}", std::env::current_exe().unwrap().display());
        debug!("Loaded config: {:?}", &self.config);
        info!(
            "Rustc version: {}, core version: {}",
            RUSTC_VERSION, CORE_VERSION
        );
        self.load_plugins().await?;
        self.init_inner_commands();
        return Ok(());
    }
    async fn shutdown(&mut self) {
        info!("Stopping main selector...");
        for (name, plugin_wrapper) in self.plugin_manager.plugins.iter() {
            // let locked_1 = plugin.lock().await;
            let mut locked = plugin_wrapper.plugin_instance.lock().await;
            // locked.on_disable().
            if let Err(e) = tokio::time::timeout(Duration::from_secs(3), locked.on_disable()).await
            {
                error!(
                    "{}: Spent more than 3s in on_disable, killing it..\n{}",
                    name, e
                );
            }
        }
        self.stop = true;
        tokio::time::sleep(Duration::from_millis(500)).await;
        std::process::exit(0);
    }
    fn init_inner_commands(&mut self) {
        self.command_manager
            .update_plugin_name(String::from("<bot>"));
        self.register_command(
            Command::new("help")
                .group(true)
                .private(true)
                .console(true)
                .description("查看帮助")
                .with_plugin_name(&String::from("<bot>")),
        )
        .ok();
        self.register_command(
            Command::new("stop")
                .console(true)
                .description("Stop the bot.")
                .with_plugin_name(&String::from("<bot>")),
        )
        .ok();
        self.register_command(
            Command::new("server_status")
                .console(true)
                .description("查询onebot服务端状态")
                .with_plugin_name(&String::from("<bot>")),
        )
        .ok();
        self.register_command(
            Command::new("server_version")
                .console(true)
                .description("查询onebot服务端版本")
                .with_plugin_name(&String::from("<bot>")),
        )
        .ok();
        self.register_command(
            Command::new("status")
                .console(true)
                .private(true)
                .group(true)
                .description("查询Bot运行状态")
                .with_plugin_name(&String::from("<bot>")),
        )
        .ok();
        self.register_command(
            Command::new("about")
                .console(true)
                .private(true)
                .group(true)
                .description("关于此Bot")
                .with_plugin_name(&String::from("<bot>")),
        )
        .ok();
    }
}
