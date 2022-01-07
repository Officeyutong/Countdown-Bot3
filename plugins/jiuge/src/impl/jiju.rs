use std::borrow::Borrow;

use countdown_bot3::countdown_bot::client::ResultType;
use log::debug;
use serde::Deserialize;

use crate::{r#impl::util::SimpleCeleryID, JiugePlugin};

use super::PoemResult;

#[derive(Deserialize, Debug, Clone)]
pub struct Source {
    pub author: String,
    pub dynasty: String,
    pub title: String,
}
#[derive(Deserialize, Debug)]
pub struct JijuResult {
    pub sens: [String; 4],
    // pub poem_id: i64,
    // pub status: String,
    pub title: String,
    pub source: [Source; 4],
}
impl PoemResult for JijuResult {
    fn to_string(&self, user_id: &str) -> String {
        let mut buf = String::new();
        buf.push_str(format!("{} 生成完成!\n", user_id).as_str());
        buf.push_str(self.title.as_str());
        buf.push_str("\n\n");
        let arr: &[String; 4] = &self.borrow().sens;
        arr.clone()
            .map(|v| buf.push_str(format!("{}\n", v).as_str()));
        buf.push_str("来源:\n");
        self.source
            .clone()
            .map(|v| buf.push_str(format!("{}-{} <{}>\n", v.dynasty, v.author, v.title).as_str()));
        return buf;
    }

    fn poem_string(&self) -> String {
        return serde_json::to_string(&self.sens).unwrap();
    }
}

impl JiugePlugin {
    pub async fn create_jiju(&self, poem: &str, user_id: &str) -> ResultType<Box<dyn PoemResult>> {
        let parsed: SimpleCeleryID = serde_json::from_str(
            self.http_client
                .post("https://jiuge.cs.tsinghua.edu.cn/api_jiju/send_jiju")
                .form(&[("poem", poem), ("user_id", user_id)])
                .send()
                .await?
                .text()
                .await?
                .as_str(),
        )?;
        debug!("Received celery ID: {}", parsed.celery_id);
        return Ok(self
            .wait_for_result::<JijuResult>(
                parsed.celery_id.as_str(),
                "https://jiuge.cs.tsinghua.edu.cn/api_jiju/get_jiju",
            )
            .await?);
    }
}
