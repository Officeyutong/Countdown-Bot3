use serde::{Deserialize, Serialize};

use crate::{
    countdown_bot::event::{
        message::{GroupSenderRole, SenderSex},
        request::GroupRequestSubType,
    },
    declare_api_call,
};

use super::CountdownBotClient;

impl CountdownBotClient {
    declare_api_call!(
        set_group_kick,
        (),
        (group_id, u64),
        (user_id, u64),
        (reject_add_request, bool)
    );
    declare_api_call!(
        set_group_ban,
        (),
        (group_id, u64),
        (user_id, u64),
        (duration, u64)
    );
    declare_api_call!(
        set_group_anonymous_ban,
        (),
        (group_id, u64),
        (anonymous_flag, &str),
        (duration, u64)
    );
    declare_api_call!(set_group_whole_ban, (), (group_id, u64), (enable, bool));
    declare_api_call!(
        set_group_admin,
        (),
        (group_id, u64),
        (user_id, u64),
        (enable, bool)
    );
    declare_api_call!(set_group_anonymous, (), (group_id, u64), (enable, bool));
    declare_api_call!(
        set_group_card,
        (),
        (group_id, u64),
        (user_id, u64),
        (card, &str)
    );
    declare_api_call!(set_group_name, (), (group_id, u64), (group_name, &str));
    declare_api_call!(set_group_leave, (), (group_id, u64), (is_dismiss, bool));
    declare_api_call!(
        set_group_special_title,
        (),
        (group_id, u64),
        (user_id, u64),
        (special_title, &str),
        (duration, i32)
    );
    declare_api_call!(
        set_group_add_request,
        (),
        (flag, &str),
        (sub_type, &GroupRequestSubType),
        (approve, bool),
        (reason, &str)
    );
    declare_api_call!(
        get_group_member_info,
        GroupMemberInfo,
        (group_id, i64),
        (user_id, i64),
        (no_cache, bool)
    );
    declare_api_call!(get_group_member_list, Vec<GroupMemberInfo>, (group_id, i64));
    declare_api_call!(
        get_group_honor_info,
        GroupHonorFetchResp,
        (group_id, i64),
        (r#type, HonorFetchType)
    );
}

#[derive(Debug, Deserialize)]
pub struct GroupMemberInfo {
    pub group_id: i64,
    pub user_id: i64,
    pub nickname: String,
    pub card: String,
    pub sex: SenderSex,
    pub age: i32,
    pub area: Option<String>,
    pub join_time: i64,
    pub last_sent_time: i64,
    pub level: String,
    pub role: GroupSenderRole,
    pub unfriendly: bool,
    pub title: Option<String>,
    pub title_expire_time: i64,
    pub card_changeable: bool,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HonorFetchType {
    Talkative,
    Performer,
    Legend,
    StrongNewbie,
    Emotion,
    All,
}
#[derive(Deserialize, Debug)]
pub struct TalkativeMemberInfo {
    pub user_id: i64,
    pub nickname: String,
    pub avatar: String,
    pub day_count: i32,
}
#[derive(Deserialize, Debug)]
pub struct GeneralMemberHonorInfo {
    pub user_id: i64,
    pub nickname: String,
    pub avatar: String,
    pub description: String,
}
#[derive(Deserialize, Debug)]
pub struct GroupHonorFetchResp {
    pub group_id: i64,
    // 当前龙王
    pub current_talkative: Option<TalkativeMemberInfo>,
    // 历史龙王
    pub talkative_list: Option<Vec<GeneralMemberHonorInfo>>,
    // 群聊之火
    pub performer_list: Option<Vec<GeneralMemberHonorInfo>>,
    // 群聊炽焰
    pub legend_list: Option<Vec<GeneralMemberHonorInfo>>,
    // 冒尖小春笋
    pub strong_newbie_list: Option<Vec<GeneralMemberHonorInfo>>,
    // 快乐之源
    pub emotion_list: Option<Vec<GeneralMemberHonorInfo>>,
}
