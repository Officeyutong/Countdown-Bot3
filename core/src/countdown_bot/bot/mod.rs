use std::collections::HashSet;
use std::path;
use std::sync::Arc;
pub type ReceiverMap = std::collections::HashMap<String, SingleCallSender>;
use super::client::{CountdownBotClient, SingleCallSender};
use super::command::SenderType;
use super::command::{Command, CommandManager, CommandSender};
use super::config::CountdownBotConfig;
use super::event::message::MessageEvent;
use super::event::EventContainer;
use super::plugin::PluginManager;
use config::Config;
use futures_util::stream::{SplitSink, SplitStream};
use log::{debug, error, info};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
pub type WriteStreamType =
    Option<SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>;
pub type ReadStreamType =
    Option<SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>;
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
    receiver_map: ReceiverMap,
    client: Option<CountdownBotClient>,
    stop_signal_sender: Option<tokio::sync::watch::Sender<bool>>,
    stop_signal_receiver: Option<tokio::sync::watch::Receiver<bool>>,
    command_manager: CommandManager,
    preserved_plugin_names: HashSet<String>,
}
mod load_plugins_impl;
mod start_impl;
impl CountdownBot {
    pub fn create_client(&self) -> CountdownBotClient {
        return self.client.as_ref().unwrap().clone();
    }
    async fn dispatch_event(&mut self, event: &EventContainer) {
        for (_, val) in self.plugin_manager.plugins.iter() {
            val.lock()
                .await
                .plugin
                .clone()
                .lock()
                .await
                .on_event(event.clone())
                .await;
        }
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
            receiver_map: ReceiverMap::new(),
            client: None,
            stop_signal_sender: None,
            stop_signal_receiver: None,
            command_manager: CommandManager::new(),
            preserved_plugin_names: HashSet::from(PRESERVED_PLUGIN_NAMES.map(|x| String::from(x))),
        }
    }
    pub async fn init(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut cfg = Config::new();
        cfg.merge(config::Config::try_from(&CountdownBotConfig::default())?)?;
        cfg.merge(config::File::with_name("config"))
            .expect("Cannot read config file!");
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
    fn shutdown(&mut self) {
        info!("Stopping main selector...");
        self.stop = true;
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
                .description("Show command help")
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
    }
    async fn on_command_stop(&mut self, _sender: &SenderType) {
        self.stop_signal_sender.as_ref().unwrap().send(true).ok();
    }
    async fn on_command_help(&mut self, sender: &SenderType) {
        let mut buf = String::from("\n");
        let cmds = self
            .command_manager
            .command_map
            .iter()
            .filter(match sender {
                SenderType::Console(_) => |p: &(&String, &Arc<Command>)| p.1.console_enabled,
                SenderType::Private(_) => |p: &(&String, &Arc<Command>)| p.1.private_enabled,
                SenderType::Group(_) => |p: &(&String, &Arc<Command>)| p.1.group_enabled,
            })
            // .map(|x|(x.0.clone(),x.1.clone()))
            .collect::<Vec<(&String, &Arc<Command>)>>();

        for (name, cmd) in cmds.iter() {
            buf.push_str(format!("{}", name).as_str());
            if !cmd.alias.is_empty() {
                buf.push_str(format!("[{}]", cmd.alias.join(",").as_str()).as_str());
            }
            buf.push_str(format!(": {}\n", cmd.description.as_str()).as_str());
        }
        match sender {
            SenderType::Console(_) => info!("{}", buf),
            SenderType::Private(_) => todo!(),
            SenderType::Group(_) => todo!(),
        }
    }
    async fn on_command(&mut self, command: String, _args: Vec<String>, sender: SenderType) {
        match command.as_str() {
            "help" => self.on_command_help(&sender).await,
            "stop" => self.on_command_stop(&sender).await,
            _ => {}
        };
    }
    async fn dispatch_command(&mut self, sender: CommandSender) {
        let parsed_sender = sender.parse_sender().unwrap();
        let mut is_console = false;
        let enable_checker: fn(&Arc<Command>) -> bool = match &parsed_sender {
            SenderType::Console(_) => {
                is_console = true;
                |v| v.console_enabled
            }
            SenderType::Private(_) => |v| v.private_enabled,
            SenderType::Group(_) => |v| v.group_enabled,
        };
        let cmd_line = match &parsed_sender {
            SenderType::Console(evt) => evt.line.clone(),
            SenderType::Private(evt) => evt.message.clone(),
            SenderType::Group(evt) => evt.message.clone(),
        };
        let splitted = cmd_line.split(" ").collect::<Vec<&str>>();
        let exec_ret: Result<(), String> =
            match self.command_manager.get_command(&String::from(splitted[0])) {
                Ok(cmd) => {
                    if enable_checker(&cmd) {
                        if cmd.plugin_name.as_ref().unwrap() == "<bot>" {
                            self.on_command(
                                cmd.command_name.clone(),
                                splitted
                                    .iter()
                                    .map(|x| String::from(*x))
                                    .collect::<Vec<String>>(),
                                parsed_sender.clone(),
                            )
                            .await;
                        } else {
                            let cmd_local = cmd.clone();
                            let plugin = (self)
                                .plugin_manager
                                .plugins
                                .get(cmd_local.plugin_name.as_ref().unwrap())
                                .unwrap()
                                .clone();
                            let cmd_name = cmd.command_name.clone();
                            let args = splitted
                                .iter()
                                .map(|x| String::from(*x))
                                .collect::<Vec<String>>();
                            let sender_cloned = parsed_sender.clone();

                            if cmd.async_command {
                                // tokio::spawn(async move {
                                //     plugin
                                //         .lock()
                                //         .await
                                //         .plugin
                                //         .lock()
                                //         .await
                                //         .on_command(cmd_name, args, sender_cloned)
                                //         .await;
                                // });
                            } else {
                                plugin
                                    .lock()
                                    .await
                                    .plugin
                                    .lock()
                                    .await
                                    .on_command(cmd_name, args, sender_cloned)
                                    .await;
                            }
                        }
                        Ok(())
                    } else {
                        if is_console {
                            Err(String::from("This command does not support console"))
                        } else {
                            Err(String::from("此指令不支持当前对话环境"))
                        }
                    }
                }
                Err(err) => Err(String::from(format!("{}", err))),
            };
        let client = self.create_client();
        if let Err(s) = exec_ret {
            match &parsed_sender {
                SenderType::Console(_) => error!("{}", s),
                SenderType::Private(evt) => {
                    client
                        .quick_send(&MessageEvent::Private(evt.clone()), &s)
                        .await
                }
                SenderType::Group(evt) => {
                    client
                        .quick_send(&MessageEvent::Group(evt.clone()), &s)
                        .await
                }
            }
        }
    }
}
