use crate::{
    common::{get_html, get_inner_text, EXPR},
    COVID19Plugin,
};
use anyhow::anyhow;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct COVNewsEntry {
    pub title: String,
    pub summary: String,
    pub source_url: String,
    pub pub_date_str: String,
    pub info_source: String,
}
impl COVID19Plugin {
    pub async fn handle_covnews(&self, sender: &SenderType) -> ResultType<()> {
        let result = {
            let html = get_html().await?;
            let script_text = get_inner_text(&html, "#getTimelineService1")?;
            let search_result = EXPR
                .find(&script_text)
                .ok_or(anyhow!("未找到相应的JSON!"))?;
            let parsed_json = serde_json::from_str::<Vec<COVNewsEntry>>(search_result.as_str())?;
            let mut result = String::from("数据来源: 丁香医生\n");
            for item in parsed_json.iter() {
                result.push_str(
                    format!(
                        r###"{} - {} - {}
{}
{}
"###,
                        item.title,
                        item.info_source,
                        item.pub_date_str,
                        item.source_url,
                        item.summary
                    )
                    .as_str(),
                );
            }
            result
        };
        self.client
            .clone()
            .unwrap()
            .quick_send_by_sender(sender, &result)
            .await?;
        return Ok(());
    }
}
