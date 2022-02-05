use std::{collections::HashSet, fmt::Debug};

use serde::{Deserialize, Serialize};
pub const SIMPLE_SALT: &str = "qwqqwqqwq";
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ZxhdmxConfig {
    pub enable_groups: HashSet<i64>,
    pub admin_password: String,
    pub min_required_players: i64,
}

impl Default for ZxhdmxConfig {
    fn default() -> Self {
        Self {
            admin_password: "qwqqwqqwq".to_string(),
            min_required_players: 2,
            enable_groups: HashSet::new(),
        }
    }
}

impl ZxhdmxConfig {
    pub fn verify_password(&self, s: &str) -> bool {
        let valid_md5 = format!(
            "{:x}",
            md5::compute(
                format!(
                    "{:x}{}",
                    md5::compute(self.admin_password.as_bytes()),
                    SIMPLE_SALT
                )
                .as_bytes()
            )
        );
        return s.to_lowercase() == valid_md5.to_lowercase();
    }
}
