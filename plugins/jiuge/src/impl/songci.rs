use countdown_bot3::countdown_bot::client::ResultType;
use log::debug;
use serde::Deserialize;

use crate::{r#impl::util::SimpleCeleryID, JiugePlugin};

use super::PoemResult;

#[derive(Deserialize, Debug)]
pub struct SongciResult {
    pub output: Vec<Vec<String>>,
    // pub poem_id: i64,
    // pub status: String,
    // pub source: [Source; 4],
    pub title: String,
}
impl PoemResult for SongciResult {
    fn to_string(&self, user_id: &str) -> String {
        let mut buf = String::new();
        buf.push_str(format!("{} 生成完成!\n", user_id).as_str());
        buf.push_str(self.title.as_str());
        buf.push_str("\n\n");
        self.output
            .iter()
            .map(|v| {
                v.iter()
                    .map(|u| buf.push_str(format!("{}\n", u).as_str()))
                    .count();
                buf.push_str("\n");
            })
            .count();
        return buf;
    }

    fn poem_string(&self) -> String {
        return serde_json::to_string(&self.output).unwrap();
    }
}

impl JiugePlugin {
    pub async fn create_songci(
        &self,
        poem: &str,
        cipai: i32,
        user_id: &str,
    ) -> ResultType<Box<dyn PoemResult>> {
        let parsed: SimpleCeleryID = serde_json::from_str(
            self.http_client
                .post("https://jiuge.cs.tsinghua.edu.cn/jiugepoem/task/send_songci")
                .form(&[
                    ("poem", poem),
                    ("user_id", user_id),
                    ("cipai", cipai.to_string().as_str()),
                ])
                .send()
                .await?
                .text()
                .await?
                .as_str(),
        )?;
        debug!("Received celery ID: {}", parsed.celery_id);
        return Ok(self
            .wait_for_result::<SongciResult>(
                parsed.celery_id.as_str(),
                "https://jiuge.cs.tsinghua.edu.cn/jiugepoem/task/get_songci",
            )
            .await?);
    }
}
