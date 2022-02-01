use anyhow::anyhow;
use countdown_bot3::countdown_bot::client::ResultType;
use log::info;
use regex::Regex;
use serde_json::Value;
lazy_static::lazy_static! {
    static ref SCRIPT_EXPR:Regex = Regex::new(r###"JSON.parse\(decodeURIComponent\("(?P<script>.+)"\)\);window._feConfigVersion="###).unwrap();
}
pub async fn fetch_luogu_pasteboard(url: &str) -> ResultType<String> {
    info!("Fetching {}", url);
    let resp = reqwest::get(url)
        .await
        .map_err(|e| anyhow!("下载网页时出错: {}", e))?
        .text()
        .await
        .map_err(|e| anyhow!("下载网页时出错: {}", e))?;
    let groups = SCRIPT_EXPR
        .captures(&resp)
        .ok_or(anyhow!("无法在网页中找到数据部分！"))?;
    let script = groups
        .name("script")
        .ok_or(anyhow!("未找到指定分组!"))?
        .as_str();
    let decoded = urlencoding::decode(script).map_err(|e| anyhow!("解码失败!\n{}", e))?;
    let parsed_json = serde_json::from_str::<Value>(&decoded)
        .map_err(|e| anyhow!("反序列化时发生错误: {}", e))?;
    let entry = parsed_json
        .pointer("/currentData/paste/data")
        .ok_or(anyhow!("找不到指定元素!"))?
        .as_str()
        .ok_or(anyhow!("指定元素不是str!"))?;
    return Ok(entry.to_string());
}
