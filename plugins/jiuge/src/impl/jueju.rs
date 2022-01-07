use countdown_bot3::countdown_bot::client::ResultType;
use log::debug;
use serde::Deserialize;

use crate::{r#impl::util::SimpleCeleryID, JiugePlugin};

use super::PoemResult;

#[derive(Deserialize, Debug)]
pub struct JuejuResult {
    pub output: [String; 4],
    // pub poem_id: i64,
    pub score: Option<[String; 4]>,
    // pub status: String,
    pub title: String,
}
impl PoemResult for JuejuResult {
    fn to_string(&self, user_id: &str) -> String {
        let mut buf = String::new();
        buf.push_str(format!("{} 生成完成!\n", user_id).as_str());
        buf.push_str(self.title.as_str());
        buf.push_str("\n\n");
        self.output
            .clone()
            .map(|v| buf.push_str(format!("{}\n", v).as_str()));
        if let Some(ref score) = self.score {
            score
                .iter()
                .zip(&["通顺性", "连贯性", "新颖性", "意境"])
                .for_each(|(n, v)| buf.push_str(format!("{}: {}\n", v, n).as_str()));
        }
        return buf;
    }

    fn poem_string(&self) -> String {
        return serde_json::to_string(&self.output).unwrap();
    }
}

impl JiugePlugin {
    pub async fn create_jueju(
        &self,
        yan: i32,
        poem: &str,
        user_id: &str,
        style: Option<i32>,
    ) -> ResultType<Box<dyn PoemResult>> {
        let parsed: SimpleCeleryID = serde_json::from_str(
            {
                match style {
                    Some(style) => self
                        .http_client
                        .post("https://jiuge.cs.tsinghua.edu.cn/jiugepoem/task/send_juejustyle")
                        .form(&[
                            ("yan", yan.to_string().as_str()),
                            ("poem", poem),
                            ("user_id", user_id),
                            ("style", style.to_string().as_str()),
                        ]),
                    None => self
                        .http_client
                        .post("https://jiuge.cs.tsinghua.edu.cn/jiugepoem/task/send_jueju")
                        .form(&[
                            ("yan", yan.to_string().as_str()),
                            ("poem", poem),
                            ("user_id", user_id),
                        ]),
                }
            }
            .send()
            .await?
            .text()
            .await?
            .as_str(),
        )?;
        debug!("Received celery ID: {}", parsed.celery_id);
        return Ok(self
            .wait_for_result::<JuejuResult>(
                parsed.celery_id.as_str(),
                match style {
                    Some(_) => "https://jiuge.cs.tsinghua.edu.cn/jiugepoem/task/get_juejustyle",
                    None => "https://jiuge.cs.tsinghua.edu.cn/jiugepoem/task/get_jueju",
                },
            )
            .await?);
    }
}
