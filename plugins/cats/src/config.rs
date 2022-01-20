use anyhow::anyhow;
use countdown_bot3::countdown_bot::command::SenderType;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
pub struct CatsConfig {
    pub image_size_limit: u64,
    pub white_list_groups: Vec<i64>,
    pub white_list_users: Vec<i64>,
    pub secret_id: String,
    pub secret_key: String,
    pub use_tencent_cloud: bool,
    pub try_delay: i64,
    pub success_delay: i64,
}

impl Default for CatsConfig {
    fn default() -> Self {
        Self {
            image_size_limit: 8 * 1024 * 1024,
            secret_id: "".to_string(),
            secret_key: "".to_string(),
            success_delay: 60 * 60,
            try_delay: 60,
            use_tencent_cloud: false,
            white_list_groups: vec![],
            white_list_users: vec![],
        }
    }
}
impl CatsConfig {
    pub fn ensure_can_upload(&self, sender: &SenderType) -> anyhow::Result<bool> {
        match sender {
            SenderType::Console(_) => todo!(),
            SenderType::Private(evt) => {
                if self.white_list_users.contains(&(evt.user_id as i64)) {
                    return Ok(false);
                }
                if !self.use_tencent_cloud {
                    return Err(anyhow!("未开启腾讯云检查，禁止上传！"));
                }
                return Ok(true);
            }
            SenderType::Group(evt) => {
                if self.white_list_groups.contains(&(evt.group_id)) {
                    return Ok(false);
                }
                if self.white_list_users.contains(&(evt.user_id as i64)) {
                    return Ok(false);
                }
                if !self.use_tencent_cloud {
                    return Err(anyhow!("未开启腾讯云检查，本群禁止上传！"));
                }
                return Ok(true);
            }
        }
    }
}
