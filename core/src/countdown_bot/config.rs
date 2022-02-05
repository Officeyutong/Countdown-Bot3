use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebServerProps {
    pub bind_ip: String,
    pub bind_port: u16,
    pub template_prefix: String,
    pub enable: bool,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountdownBotConfig {
    pub debug: bool,
    pub server_url: String,
    pub access_token: String,
    pub reconnect_interval: u32,
    pub command_prefix: Vec<String>,
    pub ignored_plugins: Vec<String>,
    pub blacklist_users: Vec<i64>,
    pub command_cooldown: u64,
    pub web_server: WebServerProps,
}
impl Default for WebServerProps {
    fn default() -> Self {
        Self {
            bind_ip: "127.0.0.1".to_string(),
            bind_port: 5001,
            template_prefix: "http://127.0.0.1:5001".to_string(),
            enable: true,
        }
    }
}
impl Default for CountdownBotConfig {
    fn default() -> CountdownBotConfig {
        CountdownBotConfig {
            debug: false,
            access_token: String::from(""),
            server_url: String::from("ws://127.0.0.1:2333"),
            reconnect_interval: 5,
            command_prefix: vec![String::from("--"), String::from("!!")],
            ignored_plugins: vec![],
            blacklist_users: vec![],
            command_cooldown: 0,
            web_server: WebServerProps::default(),
        }
    }
}
