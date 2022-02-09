use serde::Deserialize;
use serde_json::Value;

use crate::declare_api_call;

use super::{message::ComposedMessageId, CountdownBotClient};
#[derive(Deserialize)]
pub struct GuildServiceProfileResponse {
    pub nickname: String,
    pub tiny_id: String,
    pub avatar_url: String,
}
#[derive(Deserialize)]
pub struct GuildListEntry {
    pub guild_id: String,
    pub guild_name: String,
    pub guild_display_id: String,
}
#[derive(Deserialize)]
pub struct GuildMeta {
    pub guild_id: String,
    pub guild_name: String,
    pub guild_profile: String,
    pub create_time: i64,
    pub max_member_count: i64,
    pub max_robot_count: i64,
    pub max_admin_count: i64,
    pub member_count: i64,
    pub owner_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SubChannelInfo {
    pub owner_guild_id: String,
    pub channel_id: String,
    /*
    1 - 文字频道, 2 - 语音频道, 5 - 直播频道, 7 - 主题频道
     */
    pub channel_type: i32,
    pub channel_name: String,
    pub create_time: i64,
    pub creator_tiny_id: String,
    pub talk_permission: i32,
    pub visible_type: i32,
    pub current_slow_mode: i32,
    pub slow_modes: Vec<SlowModeInfo>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct SlowModeInfo {
    pub slow_mode_key: i32,
    pub slow_mode_text: String,
    pub speak_frequency: i32,
    pub slow_mode_circle: i32,
}
#[derive(Deserialize)]
pub struct GuildMemberInfo {
    pub tiny_id: String,
    pub title: String,
    pub nickname: String,
    pub role_id: String,
    pub role_name: String,
}
#[derive(Deserialize)]
pub struct GuildMemberListResponse {
    pub members: Vec<GuildMemberInfo>,
    pub finished: bool,
    pub next_token: String,
}
#[derive(Deserialize)]
pub struct GuildMemberProfile {
    pub tiny_id: String,
    pub nickname: String,
    pub avatar_url: String,
    pub join_time: i64,
    pub roles: Vec<RoleInfo>,
}
#[derive(Deserialize)]
pub struct RoleInfo {
    pub role_id: String,
    pub role_name: String,
}
#[derive(Deserialize)]
pub struct PosterInfo {
    pub tiny_id: String,
    pub nickname: String,
    pub icon_url: String,
}
#[derive(Deserialize)]
pub struct FeedMedia {
    pub file_id: String,
    pub pattern_id: String,
    pub url: String,
    pub height: i32,
    pub width: i32,
}
#[derive(Deserialize)]
pub struct FeedContent {
    pub r#type: String,
    // 偷懒，见https://github.com/Mrs4s/go-cqhttp/blob/master/docs/guild.md#%E5%86%85%E5%AE%B9%E7%B1%BB%E5%9E%8B%E5%88%97%E8%A1%A8
    pub data: Value,
}
#[derive(Deserialize)]
pub struct ResourceInfo {
    pub images: Vec<FeedMedia>,
    pub videos: Vec<FeedMedia>,
}
#[derive(Deserialize)]
pub struct FeedInfo {
    pub id: String,
    pub channel_id: String,
    pub guild_id: String,
    pub create_time: i64,
    pub title: String,
    pub sub_title: String,
    pub poster_info: PosterInfo,
    pub resource: ResourceInfo,
    pub contents: Vec<FeedContent>,
}
#[derive(Deserialize)]
pub struct ChannelMessageIdResp {
    pub message_id: String,
}
impl Into<ComposedMessageId> for ChannelMessageIdResp {
    fn into(self) -> ComposedMessageId {
        ComposedMessageId {
            message_id_i64: -1,
            message_id_str: self.message_id,
        }
    }
}
impl CountdownBotClient {
    declare_api_call!(get_guild_service_profile, GuildServiceProfileResponse,);
    declare_api_call!(get_guild_list, Option<Vec<GuildListEntry>>,);
    declare_api_call!(get_guild_meta_by_guest, GuildMeta, (guild_id, &str));
    declare_api_call!(
        get_guild_channel_list,
        Option<Vec<SubChannelInfo>>,
        (guild_id, &str),
        (no_cache, bool)
    );
    declare_api_call!(
        get_guild_member_list,
        GuildMemberListResponse,
        (guild_id, &str),
        (next_token, &str)
    );
    declare_api_call!(
        get_guild_member_profile,
        GuildMemberProfile,
        (guild_id, &str),
        (user_id, &str)
    );
    declare_api_call!(
        send_guild_channel_msg,
        ChannelMessageIdResp,
        (guild_id, &str),
        (channel_id, &str),
        (message, &str)
    );
    declare_api_call!(
        get_topic_channel_feeds,
        Vec<FeedInfo>,
        (guild_id, &str),
        (channel_id, &str)
    );
}
