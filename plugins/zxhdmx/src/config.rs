use std::{collections::HashSet, fmt::Debug};

use serde::{Deserialize, Serialize};

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
