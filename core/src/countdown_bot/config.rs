use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
pub struct CountdownBotConfig {
    pub debug: bool,
    pub server_url: String,
    pub access_token: String,
    pub reconnect_interval: u32,
}

impl CountdownBotConfig {
    pub fn default() -> CountdownBotConfig {
        CountdownBotConfig {
            debug: false,
            access_token: String::from(""),
            server_url: String::from("ws://127.0.0.1:2333"),
            reconnect_interval: 5,
        }
    }
}
