use crate::{LoginMode, MostSimpleResp, Music163Config, Music163Plugin};
use anyhow::anyhow;
use clap::ArgMatches;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use log::{error, info};
use serde::Deserialize;
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MusicSearchResponse {
    pub song_count: i32,
    pub songs: Vec<MusicSearchResponseEntry>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct MusicSearchResponseEntry {
    pub id: i64,
    pub name: String,
    pub artists: Vec<Artist>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct MusicDetail {
    pub id: i64,
    pub name: String,
    pub ar: Vec<Artist>,
}
impl MusicDetail {
    pub fn make_artist_string(&self) -> String {
        self.ar
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<String>>()
            .join(",")
    }
    pub fn make_song_string(&self) -> String {
        format!("{} ({})", self.name, self.make_artist_string())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Artist {
    pub name: String,
}
impl Music163Plugin {
    fn get_config(&self) -> &Music163Config {
        self.config.as_ref().unwrap()
    }
    pub async fn handle_command<'a>(
        &self,
        sender: &SenderType,
        args: ArgMatches<'a>,
    ) -> ResultType<()> {
        let bot_client = self.client.clone().unwrap();
        let mut send_record = args.is_present("record");
        let send_url = args.is_present("url");
        let send_share = args.is_present("share");
        let use_id = args.is_present("id");
        let keyword = args
            .value_of("KEYWORD")
            .ok_or(anyhow!("请输入查询关键字!"))?;
        let flags = [send_record, send_share, send_url];
        let mut true_count = 0;
        for x in flags {
            true_count += if x { 1 } else { 0 };
        }
        if true_count == 0 {
            send_record = true;
        } else if true_count > 1 {
            return Err(anyhow!("record,url,share三者只能选其一!").into());
        }
        if !self.check_login_status().await? {
            self.try_to_login().await?;
            if !self.check_login_status().await? {
                return Err(anyhow!("登陆失败!").into());
            }
        }
        let music_id = if use_id {
            i64::from_str_radix(keyword, 10)?
        } else {
            let search_result = self.search_music(keyword).await?;
            info!("{:?}", search_result);
            if search_result.is_empty() {
                return Err(anyhow!("搜索结果为空!").into());
            }
            search_result[0].id
        };
        if !self.check_music_exists(music_id).await? {
            return Err(anyhow!("所请求的音乐ID不存在!").into());
        }
        if send_share {
            bot_client
                .quick_send_by_sender(
                    sender,
                    format!("[CQ:music,type=163,id={}]", music_id).as_str(),
                )
                .await?;
        }
        let detail = self.get_music_detail(music_id).await?;
        let music_url = self
            .get_music_url(music_id)
            .await
            .map_err(|e| anyhow!("获取音乐下载链接时失败! :{}", e))?;
        info!("Music url: {}", music_url);
        bot_client
            .quick_send_by_sender(
                sender,
                format!("发送歌曲中: {}", detail.make_song_string()).as_str(),
            )
            .await?;
        if send_url {
            bot_client
                .quick_send_by_sender(sender, music_url.as_str())
                .await?;
        }
        if send_record {
            bot_client
                .quick_send_by_sender(
                    sender,
                    "QQ语音音质较差，同时上传录音可能需要较长时间，请等待..",
                )
                .await?;
            bot_client
                .quick_send_by_sender(sender, format!("[CQ:record,file={}]", music_url).as_str())
                .await?;
        }
        Ok(())
    }

    async fn try_to_login(&self) -> ResultType<()> {
        let config = self.get_config();
        if config.will_login {
            let resp = match config.login_mode {
                LoginMode::Phone => self
                    .http_client
                    .get(config.sub_url("/login/cellphone")?)
                    .query(&[
                        ("phone", config.phone.as_str()),
                        ("password", config.password.as_str()),
                    ]),
                LoginMode::Email => self.http_client.get(config.sub_url("/login")?).query(&[
                    ("email", config.phone.as_str()),
                    ("password", config.password.as_str()),
                ]),
            }
            .send()
            .await?
            .text()
            .await?;
            let parsed = serde_json::from_str::<MostSimpleResp>(resp.as_str())?;
            if parsed.code != 200 {
                error!("{:#?}", resp);
                return Err(anyhow!("登录失败!").into());
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    }

    async fn check_login_status(&self) -> ResultType<bool> {
        let config = self.get_config();
        if !config.will_login {
            return Ok(true);
        }
        let sub_url = config.sub_url("/login/refresh")?;
        let resp = self.http_client.clone().get(sub_url).send().await?;
        info!("Login response: {:#?}", resp);
        let resp = serde_json::from_str::<MostSimpleResp>(resp.text().await?.as_str())?;
        return Ok(resp.code == 200);
    }
    async fn check_music_exists(&self, mid: i64) -> ResultType<bool> {
        #[derive(Deserialize)]
        struct Resp {
            pub success: bool,
        }
        let config = self.config.as_ref().unwrap();
        let sub_url = config.sub_url("/check/music")?;
        let resp = self
            .http_client
            .get(sub_url)
            .query(&[("id", mid.to_string().as_str())])
            .send()
            .await?
            .text()
            .await?;
        let parsed = serde_json::from_str::<Resp>(resp.as_str())?;
        return Ok(parsed.success);
    }
    async fn get_music_url(&self, mid: i64) -> ResultType<String> {
        #[derive(Deserialize)]
        struct Resp {
            pub data: Vec<Entry>,
        }
        #[derive(Deserialize)]
        struct Entry {
            pub url: String,
        }
        let config = self.config.as_ref().unwrap();
        let sub_url = config.sub_url("/song/url")?;
        let resp = self
            .http_client
            .get(sub_url)
            .query(&[("id", mid.to_string().as_str()), ("br", "320000")])
            .send()
            .await?
            .text()
            .await?;
        let parsed = serde_json::from_str::<Resp>(resp.as_str())?;
        let url = &parsed
            .data
            .get(0)
            .ok_or(anyhow!("获取到的歌曲URL列表为空!"))?
            .url;
        return Ok(url.to_string());
    }
    async fn search_music(&self, keyword: &str) -> ResultType<Vec<MusicSearchResponseEntry>> {
        let config = self.config.as_ref().unwrap();
        let sub_url = config.sub_url("/search")?;
        let resp = self
            .http_client
            .get(sub_url)
            .query(&[
                ("keywords", keyword),
                ("limit", config.search_limit.to_string().as_str()),
            ])
            .send()
            .await?
            .text()
            .await?;
        #[derive(Deserialize, Debug)]
        struct Resp {
            pub code: i32,
            pub result: MusicSearchResponse,
            pub message: Option<String>,
        }
        let parsed = serde_json::from_str::<Resp>(resp.as_str())?;
        if parsed.code != 200 {
            error!("{:#?}", parsed);
            return Err(anyhow!(
                "搜索时发生错误: {}",
                parsed.message.unwrap_or("未知".to_string())
            )
            .into());
        }
        return Ok(parsed.result.songs);
    }
    async fn get_music_detail(&self, music_id: i64) -> ResultType<MusicDetail> {
        #[derive(Deserialize)]
        struct Resp {
            pub songs: Vec<MusicDetail>,
        }

        let config = self.config.as_ref().unwrap();
        let sub_url = config.sub_url("/song/detail")?;
        let resp = self
            .http_client
            .get(sub_url)
            .query(&[("ids", music_id.to_string().as_str())])
            .send()
            .await?
            .text()
            .await?;
        let parsed = serde_json::from_str::<Resp>(resp.as_str())?;
        let data = parsed.songs.get(0).ok_or(anyhow!("非法歌曲ID!"))?;
        let c = (*data).clone();
        return Ok(c);
    }
}
