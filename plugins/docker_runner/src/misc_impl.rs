use std::time::Duration;

use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};

use crate::{CacheEntry, CacheSourceTuple, DockerRunnerPlugin};

impl DockerRunnerPlugin {
    pub async fn handle_input(&mut self, sender: &SenderType, data: &str) -> ResultType<()> {
        let (uid, gid) = match sender {
            SenderType::Group(evt) => (evt.user_id, evt.group_id),
            _ => todo!(),
        };
        let source = CacheSourceTuple {
            uid: uid as i64,
            gid,
        };
        self.input_cache.lock().await.insert(
            source.clone(),
            CacheEntry {
                input_data: String::from(data),
                inserted_at: chrono::Local::now(),
            },
        );
        let config = self.config.as_ref().unwrap();
        let to_sleep = config.input_expire_after;
        let input_cache_cloned = self.input_cache.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(to_sleep as u64)).await;
            input_cache_cloned.lock().await.remove(&source);
        });
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender(
                sender,
                format!(
                    "缓存已成功加入！此缓存将在 {} 毫秒后失效。",
                    config.input_expire_after
                )
                .as_str(),
            )
            .await?;
        return Ok(());
    }
    pub async fn get_cache(&mut self, sender: &SenderType) -> Option<String> {
        let (uid, gid) = match sender {
            SenderType::Group(evt) => (evt.user_id, evt.group_id),
            _ => todo!(),
        };
        if let Some(val) = self.input_cache.lock().await.remove(&CacheSourceTuple {
            gid,
            uid: uid as i64,
        }) {
            let config = self.config.as_ref().unwrap();
            let time_diff = chrono::Local::now() - val.inserted_at;
            if time_diff.num_milliseconds() > config.input_expire_after {
                return None;
            }
            return Some(val.input_data);
        } else {
            return None;
        }
    }
}
