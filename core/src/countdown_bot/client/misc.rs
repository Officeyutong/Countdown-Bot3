use serde::Deserialize;
use serde_json::{json, Value};

use super::{CountdownBotClient, ResultType};
use crate::{countdown_bot::event::message::SenderSex, declare_api_call};
use anyhow::anyhow;
#[derive(Deserialize, Debug)]
pub struct LoginInfo {
    pub user_id: u64,
    pub nickname: String,
}
#[derive(Deserialize, Debug)]
pub struct StrangerInfo {
    pub user_id: i64,
    pub nickname: String,
    pub sex: SenderSex,
    pub age: i32,
}
#[derive(Deserialize, Debug)]
pub struct FriendListEntry {
    pub user_id: i64,
    pub nickname: String,
    pub remark: String,
}
#[derive(Deserialize, Debug)]
pub struct GroupInfo {
    pub group_id: i64,
    pub group_name: String,
    pub member_count: i32,
    pub max_member_count: i32,
}

impl CountdownBotClient {
    declare_api_call!(send_like, (), (user_id, u64), (times, u32));
    declare_api_call!(
        set_friend_add_request,
        (),
        (flag, &str),
        (approve, bool),
        (remark, &str)
    );
    declare_api_call!(get_login_info, LoginInfo,);
    declare_api_call!(
        get_stranger_info,
        StrangerInfo,
        (user_id, i64),
        (no_cache, bool)
    );
    declare_api_call!(get_friend_list, Vec<FriendListEntry>,);
    declare_api_call!(get_group_info, GroupInfo, (group_id, i64), (no_cache, bool));
    declare_api_call!(get_group_list, Vec<GroupInfo>,);
    declare_api_call!(get_cookies, GetCredentialsResp, (domain, &str));
    declare_api_call!(get_csrf_token, GetCredentialsResp,);
    declare_api_call!(get_credentials, GetCredentialsResp,);
    declare_api_call!(get_record, SingleFileResp, (file, &str), (out_format, &str));
    // declare_api_call!(get_image, SingleFileResp, (file, &str));
    declare_api_call!(can_send_image, SingleBoolResp,);
    declare_api_call!(can_send_record, SingleBoolResp,);
    pub async fn get_status(&self) -> ResultType<BotStatus> {
        let resp = self.call("get_status", &json!({})).await?;
        Ok(BotStatus {
            good: (&resp)
                .get("good")
                .ok_or(anyhow!("Missing online field!"))?
                .as_bool()
                .unwrap(),
            online: (&resp)
                .get("online")
                .ok_or(anyhow!("Missing online field!"))?
                .as_bool()
                .unwrap(),
            value: resp,
        })
    }
    pub async fn get_version_info(&self) -> ResultType<BotVersionInfo> {
        let resp = self.call("get_version_info", &json!({})).await?;
        Ok(BotVersionInfo {
            app_name: (&resp)
                .get("app_name")
                .ok_or(anyhow!("Missing app_name field!"))?
                .as_str()
                .unwrap()
                .to_string(),
            app_version: (&resp)
                .get("app_version")
                .ok_or(anyhow!("Missing app_version field!"))?
                .as_str()
                .unwrap()
                .to_string(),
            protocol_version: (&resp)
                .get("protocol_version")
                .ok_or(anyhow!("Missing protocol_version field!"))?
                .as_str()
                .unwrap()
                .to_string(),
            value: resp,
        })
    }

    declare_api_call!(set_restart, (), (delay, i32));
    declare_api_call!(clean_cache, (),);
}

#[derive(Deserialize, Debug)]
pub struct GetCredentialsResp {
    pub cookies: Option<String>,
    pub token: Option<String>,
}
#[derive(Deserialize, Debug)]
pub struct SingleFileResp {
    pub file: String,
}
#[derive(Deserialize, Debug)]
pub struct SingleBoolResp {
    pub yes: bool,
}
#[derive(Debug)]
pub struct BotStatus {
    pub value: Value,
    pub online: bool,
    pub good: bool,
}

#[derive(Debug)]
pub struct BotVersionInfo {
    pub app_name: String,
    pub app_version: String,
    pub protocol_version: String,
    pub value: Value,
}
