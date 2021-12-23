use crate::countdown_bot::client::{APICallRequest, APICallResponse};
use crate::countdown_bot::plugin::BotPlugin;
use anyhow::anyhow;
use serde_json::Value;
use std::{path, rc::Rc};
use tokio::sync::mpsc;
pub type ReceiverMap = std::collections::HashMap<String, SingleCallSender>;
use super::client::{CountdownBotClient, SingleCallSender};
use super::config::CountdownBotConfig;
use super::plugin::PluginManager;
use config::Config;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, error, info};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub type WriteStreamType =
    Option<SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>;
pub type ReadStreamType =
    Option<SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>;
pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

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
}

impl CountdownBot {
    pub fn get_logger(&self) -> &'static dyn log::Log {
        return self.logger.unwrap();
    }
    pub fn get_max_log_level(&self) -> log::LevelFilter {
        return self.max_log_level.unwrap().clone();
    }
    pub fn echo(&self, s: &String) {
        info!("Echo: {}", s);
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
            receiver_map: ReceiverMap::new(),
            client: None,
        }
    }
    pub fn init(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
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
        info!("Starting Countdown-Bot3 ...");
        info!("Currently working path: {}", self.sys_root.display());
        info!("Executable: {}", std::env::current_exe().unwrap().display());
        debug!("Loaded config: {:?}", &self.config);
        info!(
            "Rustc version: {}, core version: {}",
            RUSTC_VERSION, CORE_VERSION
        );
        self.load_plugins()?;
        self.init_events();

        return Ok(());
    }
    pub async fn start(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use tokio_tungstenite::connect_async;
        use url::Url;
        let url = {
            let mut local = Url::parse(&self.config.server_url)?;
            local.set_query(Some(
                format!("access_token={}", self.config.access_token).as_str(),
            ));
            local
        };
        let (call_tx, mut call_rx) = mpsc::unbounded_channel::<APICallRequest>();
        fn construct_json(action: String, params: Value, token: String) -> String {
            return serde_json::to_string(&serde_json::json!({
                "action":action,
                "params":params,
                "echo":token
            }))
            .unwrap();
        }
        while !self.stop {
            match connect_async(url.clone()).await {
                Ok((stream, resp)) => {
                    info!("Connected! {}", resp.status());
                    let (write, read) = stream.split();
                    self.write_stream = Some(write);
                    self.read_stream = Some(read);
                    self.client = Some(CountdownBotClient::new(call_tx.clone()));
                    loop {
                        debug!("Selecting..");
                        tokio::select! {
                            ret = self.read_stream.as_mut().unwrap().next() => {
                                if let Some(result) = ret{
                                    match result {
                                        Ok(message) => {
                                            let raw_string = message.to_string();
                                            match serde_json::from_str::<serde_json::Value>(
                                                &raw_string.as_str(),
                                            ) {
                                                Ok(json) => {
                                                    if json.as_object().unwrap().contains_key("post_type") {
                                                        let _event =
                                                        super::event::EventContainer::from_json(&json);
                                                    } else {
                                                        if let Ok(parse_result) = serde_json::from_value::<APICallResponse>(json.clone()) {
                                                            if let Some(sender) = self.receiver_map.remove(&parse_result.echo) {
                                                                sender.send(match parse_result.status.as_str() {
                                                                    "ok" => Ok(parse_result.data),
                                                                    "failed" => Err(Box::from(anyhow!(
                                                                        "Failed to perform API call: {}",
                                                                        parse_result.retcode
                                                                    ))),
                                                                    "async" => Ok(serde_json::json!({})),
                                                                    _ => Err(Box::from(anyhow!(
                                                                        "Invalid status: {}",
                                                                        parse_result.status
                                                                    ))),
                                                                }).ok();
                                                            }
                                                        } else {
                                                            error!("Invalid call response: {:?}", &json);
                                                        }
                                                    }

                                                }
                                                Err(err) => {
                                                    error!("Invalid json! {}", err);
                                                    break;
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            error!("Error occurred: {}", err);
                                            break;
                                        }
                                    }
                                } else {
                                    error!("Empty packet received, disconnecting..");
                                }
                            }
                            call_req = call_rx.recv() => {
                                if let Some(req) = call_req{
                                    // debug!("Call request: {:?}",req);
                                    self.receiver_map.insert(req.token.clone(), req.sender);
                                    if let Err(err) = self
                                        .write_stream
                                        .as_mut()
                                        .unwrap()
                                        .send(Message::Text(construct_json(
                                            req.action.clone(),
                                            req.payload.clone(),
                                            req.token.clone(),
                                        )))
                                        .await
                                    {
                                        if let Some(r) = self.receiver_map.remove(&req.token){
                                            if let Ok(_) = r.send(Err(Box::from(anyhow!("Sending error! {}", err)))){

                                            }
                                        }
                                    }
                                }
                            }
                            // _ = tokio::time::sleep(Duration::from_secs(3)) => {
                            //     // let client = self.client.unwrap().clone();
                            //     let local_call_tx = call_tx.clone();
                            //     tokio::spawn(async move {
                            //         let mut client = CountdownBotClient::new(local_call_tx);
                            //         debug!("Sending..");
                            //         debug!("Response: {:?}", client.call("get_status", &json!({})).await);
                            //     });
                            // }
                        };
                    }
                }
                Err(err) => {
                    error!("Error occurred: {}", err);
                    info!(
                        "Reconnecting after {} seconds..",
                        self.config.reconnect_interval
                    );
                    tokio::time::sleep(core::time::Duration::from_secs(
                        self.config.reconnect_interval.into(),
                    ))
                    .await;
                }
            }
        }

        return Ok(());
    }
    fn load_plugins(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use std::env::consts::{DLL_EXTENSION, DLL_PREFIX};
        let path1 = {
            let exe = std::env::current_exe()?;
            std::path::PathBuf::from(exe.parent().unwrap())
        };
        let path2 = self.sys_root.clone();
        let load_path: Vec<std::path::PathBuf> = vec![path1, path2];
        let mut libs: Vec<std::path::PathBuf> = vec![];
        for path in load_path.iter() {
            info!("Listing libraries under: {}", path.display());
            for file in std::fs::read_dir(path).unwrap() {
                if let Ok(f) = file {
                    let lib_path = f.path();
                    if let Some(file_name) = lib_path.file_name() {
                        if let Some(s) = file_name.to_str() {
                            if let Ok(tp) = f.file_type() {
                                if !tp.is_file() {
                                    debug!("Ignoring \"{}\", it's not a file.", s);
                                    continue;
                                }
                            }
                            if s.ends_with(DLL_EXTENSION) && s.starts_with(DLL_PREFIX) {
                                libs.push(f.path());
                                debug!("Catched \"{}\"", s);
                            } else {
                                debug!("Ignoring \"{}\", because its pathname doesn't begin with \"{}\" or end with \"{}\"",s,DLL_PREFIX,DLL_EXTENSION);
                            }
                        }
                    }
                }
            }
        }
        info!("Plugins to load:");
        for item in libs.iter() {
            info!("{}", item.display());
        }
        for item in libs.iter() {
            if let Err(e) = unsafe { self.plugin_manager.load_plugin(item.as_os_str()) } {
                error!("Error loading: {}", item.display());
                error!("{}", e);
            }
        }
        let mut plugins: Vec<(String, Rc<dyn BotPlugin>)> = vec![];
        for (name, plugin) in self.plugin_manager.plugins.iter() {
            plugins.push((String::from(name), plugin.clone() as Rc<dyn BotPlugin>));
        }
        for (name, plugin) in plugins.iter() {
            info!("Loading {}", name);
            if let Err(e) = plugin.on_enable(self) {
                error!("Error enablng: {}", name);
                error!("{}", e);
            }
        }
        return Ok(());
    }
    fn init_events(&mut self) {}
}
