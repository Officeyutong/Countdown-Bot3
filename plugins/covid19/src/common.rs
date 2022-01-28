use anyhow::anyhow;
use countdown_bot3::countdown_bot::client::ResultType;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
lazy_static! {
    pub static ref EXPR: Regex = Regex::new(r"(\[.+\])").unwrap();
}
pub async fn get_html() -> ResultType<Html> {
    let html_text = reqwest::get("https://ncov.dxy.cn/ncovh5/view/pneumonia")
        .await?
        .text()
        .await?;
    let parsed = Html::parse_document(&html_text);
    return Ok(parsed);
}
pub trait ThingsThatCanMakeMessage {
    fn make_message(&self) -> String;
}
pub fn get_inner_text(html: &Html, id: &str) -> ResultType<String> {
    let elems = html
        .select(&Selector::parse(id).map_err(|e| anyhow!("非法选择器: {}\n{:?}", id, e))?)
        .collect::<Vec<ElementRef>>();
    if elems.is_empty() {
        return Err(anyhow!("无法找到{}元素", id).into());
    }
    let inner_text = elems[0]
        .text()
        .collect::<Vec<&str>>()
        .join("")
        .replace("\n", "");
    return Ok(inner_text);
}
