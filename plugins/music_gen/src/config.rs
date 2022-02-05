use std::collections::HashMap;

use serde::{Deserialize, Serialize};
// #[derive(Deserialize, Serialize, Clone)]
// pub struct DownloadInfo {
    
// }
// impl Default for DownloadInfo {
//     fn default() -> Self {
//         Self {
//             template_url: "http://127.0.0.1:5001/music_gen/download/[hash]".to_string(),
//         }
//     }
// }
#[derive(Deserialize, Serialize, Clone)]
pub struct MusicGenConfig {
    pub default_bpm: u64,
    pub max_notes: u64,
    pub max_notes_through_message: u64,
    pub default_volume: f64,
    pub redis_uri: String,
    pub cache_timeout: u64,
    pub max_storing_files: u64,
    pub group_limits: HashMap<i64, i64>,
    pub max_length_in_seconds: u64,
    pub max_execute_sametime: u64,
    pub use_cache: bool,
    // pub download: DownloadInfo,
}
impl Default for MusicGenConfig {
    fn default() -> Self {
        Self {
            default_bpm: 120,
            max_notes: 500,
            max_notes_through_message: 30,
            default_volume: 1.0,
            cache_timeout: 3 * 60,
            max_storing_files: 10,
            group_limits: Default::default(),
            max_length_in_seconds: 6 * 60,
            redis_uri: String::from("redis://127.0.0.1/0"),
            max_execute_sametime: 2,
            use_cache: false,
            // download: DownloadInfo::default(),
        }
    }
}
