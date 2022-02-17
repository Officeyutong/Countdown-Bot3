use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageSenderConfig {
    pub whitelist_users: Vec<i64>,
}

impl Default for MessageSenderConfig {
    fn default() -> Self {
        Self {
            whitelist_users: vec![-1],
        }
    }
}
