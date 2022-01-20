use crate::CatsPlugin;
use anyhow::anyhow;
use chrono::NaiveDateTime;
use countdown_bot3::countdown_bot::client::ResultType;
use hmac::{Hmac, Mac};
use log::debug;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, str::FromStr};
type HmacSha256 = Hmac<Sha256>;
fn hmac_sha256(key: &[u8], value: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(value);
    let result = mac.finalize().into_bytes();
    return result.to_vec();
}
fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    return hex::encode(hasher.finalize().as_slice());
}
pub fn make_header(
    payload: &str,
    timestamp: i64,
    secret_id: &str,
    secret_key: &str,
) -> HashMap<String, String> {
    let date = NaiveDateTime::from_timestamp(timestamp, 0)
        .format("%Y-%m-%d")
        .to_string();
    let mut request = String::new();
    request.push_str("POST\n");
    request.push_str("/\n");
    request.push_str("\n");
    request.push_str("content-type:application/json\nhost:tiia.tencentcloudapi.com\n\n");
    request.push_str("content-type;host\n");
    request.push_str(&sha256(payload.as_bytes()));
    let scope = format!("{}/tiia/tc3_request", date);
    let string_to_sign = format!(
        "TC3-HMAC-SHA256\n{}\n{}\n{}",
        timestamp,
        scope,
        sha256(request.as_bytes())
    );
    let secret_date = hmac_sha256(format!("TC3{}", secret_key).as_bytes(), date.as_bytes());
    let secret_service = hmac_sha256(&secret_date, b"tiia");
    let secret_signing = hmac_sha256(&secret_service, b"tc3_request");

    let signature = hex::encode(hmac_sha256(&secret_signing, string_to_sign.as_bytes()));
    let authorization = format!(
        "TC3-HMAC-SHA256 Credential={}/{}, SignedHeaders=content-type;host, Signature={}",
        secret_id, scope, signature
    );
    return HashMap::from_iter(
        HashMap::from([
            ("X-TC-Action", "DetectLabel"),
            ("X-TC-Version", "2019-05-29"),
            ("X-TC-Region", "ap-beijing"),
            ("X-TC-Timestamp", timestamp.to_string().as_str()),
            ("Host", "tiia.tencentcloudapi.com"),
            ("Content-Type", "application/json"),
            ("Authorization", authorization.as_str()),
        ])
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string())),
    );
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AlbumLabelEntry {
    pub name: String,
    pub first_category: String,
    pub second_category: String,
    pub confidence: i32,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RecognizeResponseA {
    pub response: RecognizeResponseB,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RecognizeResponseB {
    pub album_labels: Option<Vec<AlbumLabelEntry>>,
    pub error: Option<Value>,
}

// pub struct RecognizeResult {
//     pub ok: bool,
//     pub message: String,
// }

impl CatsPlugin {
    pub async fn recognize_cat_image(&self, image: &[u8]) -> ResultType<String> {
        let config = self.config.as_ref().unwrap();
        let timestamp = chrono::offset::Local::now().timestamp();
        let payload = serde_json::to_string(&json!( {
            "ImageBase64": base64::encode(image),
            "Scenes":["ALBUM"]
        }))?;
        let headers = make_header(&payload, timestamp, &config.secret_id, &config.secret_key);
        // let client = reqwest::ClientBuilder::new().default_headers(HeaderMap::from_iter(headers.iter().map(|(k,v)|{

        //     return (HeaderName::from_lowercase(k.to_lowercase().as_bytes()).unwrap(),v.clone());
        // }))).build()?;
        let response_text = reqwest::Client::new()
            .post("https://tiia.tencentcloudapi.com")
            .headers({
                let mut header_map = HeaderMap::new();
                headers.iter().for_each(|(k, v)| {
                    header_map.insert(
                        HeaderName::from_str(k).unwrap(),
                        HeaderValue::from_str(v).unwrap(),
                    );
                });
                header_map
            })
            .body(payload)
            .send()
            .await
            .map_err(|e| anyhow!("访问API时发生错误: {}", e))?
            .text()
            .await
            .map_err(|e| anyhow!("下载response时发生错误: {}", e))?;
        debug!("Response text: {}", response_text);
        let resp = serde_json::from_str::<RecognizeResponseA>(&response_text)?.response;
        if let Some(val) = resp.error {
            return Err(anyhow!("识别发生错误: {}", val).into());
        }
        let album_labels = resp.album_labels.ok_or(anyhow!("返回的识别结果为空!"))?;
        let mut message = String::new();
        let mut cat_recognized = false;
        album_labels.iter().for_each(|entry| {
            message.push_str(&format!(
                "识别到{}, 可信度{}%\n",
                entry.name, entry.confidence
            ));
            if entry.name.contains("猫") && entry.confidence >= 30 {
                cat_recognized = true;
            }
        });
        if cat_recognized {
            return Ok(format!("识别成功！\n{}", message));
        } else {
            return Err(anyhow!("没有识别到猫！\n{}", message).into());
        }
    }
}
