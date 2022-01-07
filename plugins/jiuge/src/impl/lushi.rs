use std::borrow::Borrow;

use countdown_bot3::countdown_bot::client::ResultType;
use log::debug;
use serde::Deserialize;

use crate::{r#impl::util::SimpleCeleryID, JiugePlugin};

use super::PoemResult;
#[derive(Deserialize, Debug)]
pub struct LushiResult {
    pub output: [String; 4],
    // pub poem_id: i64,
    // pub status: String,
    pub title: String,
}
impl PoemResult for LushiResult {
    fn to_string(&self, user_id: &str) -> String {
        let mut buf = String::new();
        buf.push_str(format!("{} 生成完成!\n", user_id).as_str());
        buf.push_str(self.title.as_str());
        buf.push_str("\n\n");
        let arr: &[String; 4] = &self.borrow().output;
        arr.clone()
            .map(|v| buf.push_str(format!("{}\n", v).as_str()));
        return buf;
    }

    fn poem_string(&self) -> String {
        return serde_json::to_string(&self.output).unwrap();
    }
}

impl JiugePlugin {
    pub async fn create_lushi(
        &self,
        yan: i32,
        poem: &str,
        user_id: &str,
    ) -> ResultType<Box<dyn PoemResult>> {
        let parsed: SimpleCeleryID = serde_json::from_str(
            self.http_client
                .post("https://jiuge.cs.tsinghua.edu.cn/jiugepoem/task/send_lvshi")
                .form(&[
                    ("yan", yan.to_string().as_str()),
                    ("poem", poem),
                    ("user_id", user_id),
                ])
                .send()
                .await?
                .text()
                .await?
                .as_str(),
        )?;
        debug!("Received celery ID: {}", parsed.celery_id);
        return Ok(self
            .wait_for_result::<LushiResult>(
                parsed.celery_id.as_str(),
                "https://jiuge.cs.tsinghua.edu.cn/jiugepoem/task/get_lvshi",
            )
            .await?);
    }
}
