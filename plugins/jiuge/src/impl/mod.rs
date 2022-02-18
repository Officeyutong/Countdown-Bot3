use std::time::Duration;

use crate::JiugePlugin;
use anyhow::anyhow;
use clap::ArgMatches;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use log::{debug, info};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use self::util::{create_uid, GeneralResponse, GetKeywordResponse};
mod arousic;
mod jiju;
mod jueju;
mod lushi;
mod songci;
mod util;
// macro_rules! decl_wait_match{
//     ($genre:expr,$config:expr,$(($id:expr,$fut:expr)),*) => {
//         match $genre {
//             $($id => tokio::time::timeout(Duration::from_secs($config.time_limit.into()),$fut).await,)*
//             _ => todo!()
//         }
//     }
// }

pub trait PoemResult {
    fn to_string(&self, user_id: &str) -> String;
    fn poem_string(&self) -> String;
}

impl JiugePlugin {
    pub async fn handle_command<'a>(
        &self,
        sender: &SenderType,
        args: &ArgMatches<'a>,
    ) -> ResultType<()> {
        let client = self.client.clone().unwrap();
        let config = self.config.as_ref().unwrap();
        let keyword = args.value_of("KEYWORD").ok_or(anyhow!("请输入关键字!"))?;
        let enable_image = args.is_present("image");
        let genre = match args.value_of("genre") {
            Some(v) => {
                i32::from_str_radix(v, 10).map_err(|e| anyhow!("解析genre时发生错误: {}", e))?
            }
            None => 1,
        };
        let yan = match args.value_of("yan") {
            Some(v) => {
                i32::from_str_radix(v, 10).map_err(|e| anyhow!("解析yan时发生错误: {}", e))?
            }
            None => 5,
        };
        if yan != 5 && yan != 7 {
            return Err(anyhow!("非法言数: {}", yan).into());
        }
        let style = match args.value_of("style") {
            Some(v) => {
                i32::from_str_radix(v, 10).map_err(|e| anyhow!("解析style时发生错误: {}", e))?
            }
            None => 0,
        };
        if ![1, 4, 2, 7, 5, 3].contains(&genre) {
            return Err(anyhow!("非法体裁: {}", genre).into());
        }
        let keyword_resp = self.get_keyword("1", genre, keyword).await?;
        if keyword_resp.code != "0" || keyword_resp.data.is_empty() {
            return Err(anyhow!("获取keyword失败!").into());
        }
        let real_keyword = keyword_resp.data[0].ch.clone();
        let user_id = create_uid();
        let ref_uid = user_id.as_str();
        let poem = real_keyword.as_str();
        let (user_message, poem_string) = {
            let wait_result: Box<dyn PoemResult> = match genre {
                1 => {
                    tokio::time::timeout(
                        Duration::from_secs(config.time_limit.into()),
                        self.create_jueju(yan, poem, ref_uid, None),
                    )
                    .await
                }
                4 => {
                    tokio::time::timeout(
                        Duration::from_secs(config.time_limit.into()),
                        self.create_jueju(yan, poem, ref_uid, Some(style)),
                    )
                    .await
                }
                2 => {
                    tokio::time::timeout(
                        Duration::from_secs(config.time_limit.into()),
                        self.create_arousic(yan, poem, style, ref_uid),
                    )
                    .await
                }
                7 => {
                    tokio::time::timeout(
                        Duration::from_secs(config.time_limit.into()),
                        self.create_lushi(yan, poem, ref_uid),
                    )
                    .await
                }
                5 => {
                    tokio::time::timeout(
                        Duration::from_secs(config.time_limit.into()),
                        self.create_jiju(poem, ref_uid),
                    )
                    .await
                }
                3 => {
                    tokio::time::timeout(
                        Duration::from_secs(config.time_limit.into()),
                        self.create_songci(poem, style, ref_uid),
                    )
                    .await
                }
                _ => todo!(),
            }
            .map_err(|e| anyhow!("生成 {} 时超时! : {}", user_id.as_str(), e))?
            .map_err(|e| anyhow!("生成 {} 时发生错误: {}", user_id.as_str(), e))?;
            (wait_result.to_string(ref_uid), wait_result.poem_string())
        };
        client
            .quick_send_by_sender(&sender, user_message.as_str())
            .await?;
        if enable_image {
            let text_resp = self
                .http_client
                .post("http://jiuge.thunlp.org/share")
                .form(&[
                    ("style", style.to_string().as_str()),
                    ("genre", genre.to_string().as_str()),
                    ("yan", yan.to_string().as_str()),
                    ("keywords", real_keyword.as_str()),
                    ("user_poem", poem_string.as_str()),
                    ("lk", ""),
                ])
                .send()
                .await?
                .text()
                .await?;
            #[derive(Deserialize, Debug)]
            struct Resp {
                pub data: String,
            }
            let parsed = serde_json::from_str::<Resp>(&text_resp)?;
            info!("Image generating response: {:?}", parsed);
            client
                .quick_send_by_sender_ex(
                    &sender,
                    format!(
                        "[CQ:image,file=http://jiuge.thunlp.org/share/new/{}]",
                        parsed.data
                    )
                    .as_str(),
                    false,
                )
                .await?;
        }
        Ok(())
    }
    async fn get_keyword(
        &self,
        level: &str,
        genre: i32,
        keywords: &str,
    ) -> ResultType<GeneralResponse<Vec<GetKeywordResponse>>> {
        let config = self.config.as_ref().unwrap();
        let sub_url = config.sub_url("/getKeyword")?;
        let resp_text = self
            .http_client
            .post(sub_url)
            .form(&[
                ("level", level),
                ("genre", genre.to_string().as_str()),
                ("keywords", keywords),
            ])
            .send()
            .await?
            .text()
            .await?;
        return Ok(serde_json::from_str(resp_text.as_str())?);
    }

    async fn wait_for_result<'de, T>(
        &self,
        celery_id: &str,
        response_url: &str,
    ) -> ResultType<Box<T>>
    where
        T: PoemResult + DeserializeOwned,
    {
        let config = self.config.as_ref().unwrap();

        let mut retried = 0;
        let poem_received = loop {
            let text_resp = self
                .http_client
                .post(response_url)
                .form(&[("celery_id", celery_id)])
                .send()
                .await?
                .text()
                .await?;
            debug!("Text response: {}", text_resp);
            let resp = serde_json::from_str::<Value>(text_resp.as_str())?;
            info!("Received: {}", resp);
            let status = resp
                .as_object()
                .ok_or(anyhow!("返回值格式错误!"))?
                .get("status")
                .ok_or(anyhow!("缺少status项!"))?
                .as_str()
                .ok_or(anyhow!("无法转换status为str!"))?;
            if status == "PENDING" {
                retried += 1;
                if retried >= config.retry_times {
                    return Err(anyhow!("重试次数过多!").into());
                };
                tokio::time::sleep(Duration::from_millis(1500)).await;
                continue;
            } else if status == "SUCCESS" {
                break serde_json::from_value::<T>(resp)?;
            } else {
                return Err(anyhow!("非法状态: {}", status).into());
            }
        };
        return Ok(Box::new(poem_received));
    }
}
