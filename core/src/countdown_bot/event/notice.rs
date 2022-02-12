use std::error::Error;

use anyhow::anyhow;
use countdown_bot_proc_macro::impl_upcast;
use serde::Deserialize;
use serde_json::{from_value, Value};

use crate::countdown_bot::client::guild::SubChannelInfo;

use super::{AbstractEvent, UnknownEvent};
#[derive(Deserialize, Debug, Clone)]
#[impl_upcast(AbstractEvent)]
pub enum NoticeEvent {
    GroupFileUpload(GroupFileUploadEvent),
    GroupAdminChange(GroupAdminChangeEvent),
    GroupMembersReduce(GroupMembersReduceEvent),
    GroupMembersIncrease(GroupMembersIncreaseEvent),
    GroupMute(GroupMuteEvent),
    FriendAdd(FriendAddEvent),
    GroupMessageRecall(GroupMessageRecallEvent),
    FriendMessageRecall(FriendMessageRecallEvent),
    GroupPoke(GroupPokeEvent),
    GroupRedbagLuckKing(GroupRedbagLuckKingEvent),
    GroupMemberHonorChange(GroupMemberHonorChangeEvent),
    GuildMessageReactionsUpdatedEvent(MessageReactionsUpdatedEvent),
    GuildSubchannelMessageUpdated(SubChannelMessageUpdatedEvent),
    GuildSubchannelCreated(SubChannelCreatedEvent),
    GuildSubchannelDestroyed(SubChannelDestroyedEvent),
    Unknown,
}
impl NoticeEvent {
    pub fn from_json(json: &Value) -> Result<NoticeEvent, Box<dyn Error>> {
        if let Value::Object(val) = json {
            let t = json.clone();
            return Ok(
                match val
                    .get("notice_type")
                    .ok_or(anyhow!("Missing 'notice_type'"))?
                    .as_str()
                    .ok_or("Expected string for 'notice_type'")?
                {
                    "group_upload" => {
                        NoticeEvent::GroupFileUpload(from_value::<GroupFileUploadEvent>(t)?)
                    }
                    "group_admin" => {
                        NoticeEvent::GroupAdminChange(from_value::<GroupAdminChangeEvent>(t)?)
                    }
                    "group_decrease" => {
                        NoticeEvent::GroupMembersReduce(from_value::<GroupMembersReduceEvent>(t)?)
                    }
                    "group_increase" => NoticeEvent::GroupMembersIncrease(from_value::<
                        GroupMembersIncreaseEvent,
                    >(t)?),
                    "group_ban" => NoticeEvent::GroupMute(from_value::<GroupMuteEvent>(t)?),
                    "friend_add" => NoticeEvent::FriendAdd(from_value::<FriendAddEvent>(t)?),
                    "group_recall" => {
                        NoticeEvent::GroupMessageRecall(from_value::<GroupMessageRecallEvent>(t)?)
                    }
                    "friend_recall" => {
                        NoticeEvent::FriendMessageRecall(from_value::<FriendMessageRecallEvent>(t)?)
                    }
                    "poke" => NoticeEvent::GroupPoke(from_value::<GroupPokeEvent>(t)?),
                    "lucky_king" => {
                        NoticeEvent::GroupRedbagLuckKing(from_value::<GroupRedbagLuckKingEvent>(t)?)
                    }
                    "honor" => NoticeEvent::GroupMemberHonorChange(from_value::<
                        GroupMemberHonorChangeEvent,
                    >(t)?),
                    "message_reactions_updates" => {
                        NoticeEvent::GuildMessageReactionsUpdatedEvent(from_value(t)?)
                    }
                    "channel_updated" => NoticeEvent::GuildSubchannelMessageUpdated(from_value(t)?),
                    "channel_created" => NoticeEvent::GuildSubchannelCreated(from_value(t)?),
                    "channel_destroyed" => NoticeEvent::GuildSubchannelDestroyed(from_value(t)?),
                    _ => NoticeEvent::Unknown,
                },
            );
        } else {
            return Err(Box::from(anyhow!("Expected JSON object!")));
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GroupFileUploadEvent {
    pub group_id: i64,
    pub user_id: i64,
    pub file: GroupFileInfo,
}
impl AbstractEvent for GroupFileUploadEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupFileInfo {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub busid: i64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GroupAdminChangeSubType {
    Set,
    Unset,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupAdminChangeEvent {
    pub sub_type: GroupAdminChangeSubType,
    pub group_id: i64,
    pub user_id: i64,
}
impl AbstractEvent for GroupAdminChangeEvent {}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GroupMembersReduceSubType {
    Leave,
    Kick,
    KickMe,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupMembersReduceEvent {
    pub sub_type: GroupMembersReduceSubType,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
}
impl AbstractEvent for GroupMembersReduceEvent {}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GroupMembersIncreaseSubtype {
    Approve,
    Invite,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GroupMembersIncreaseEvent {
    pub sub_type: GroupMembersIncreaseSubtype,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
}
impl AbstractEvent for GroupMembersIncreaseEvent {}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GroupMuteSubType {
    Ban,
    LiftBan,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GroupMuteEvent {
    pub sub_type: GroupMuteSubType,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
    pub duration: i64,
}
impl AbstractEvent for GroupMuteEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct FriendAddEvent {
    pub user_id: i64,
}
impl AbstractEvent for FriendAddEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupMessageRecallEvent {
    pub group_id: i64,
    pub user_id: i64,
    pub operator_id: i64,
    pub message_id: i64,
}
impl AbstractEvent for GroupMessageRecallEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct FriendMessageRecallEvent {
    pub user_id: i64,
    pub message_id: i64,
}
impl AbstractEvent for FriendMessageRecallEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupPokeEvent {
    pub group_id: i64,
    pub user_id: i64,
    pub target_id: i64,
}
impl AbstractEvent for GroupPokeEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupRedbagLuckKingEvent {
    pub group_id: i64,
    pub user_id: i64,
    pub target_id: i64,
}
impl AbstractEvent for GroupRedbagLuckKingEvent {}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GroupMemberHonorChangeSubType {
    Talkative,
    Performer,
    Emotion,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupMemberHonorChangeEvent {
    pub group_id: i64,
    pub honor_type: GroupMemberHonorChangeSubType,
    pub user_id: i64,
}
impl AbstractEvent for GroupMemberHonorChangeEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct ReactionInfo {
    pub emoji_id: String,
    pub emoji_index: String,
    pub emoji_type: i32,
    pub emoji_name: String,
    pub count: i32,
    pub clicked: bool,
}
#[derive(Deserialize, Debug, Clone)]
pub struct MessageReactionsUpdatedEvent {
    pub guild_id: String,
    pub channel_id: String,
    pub user_id: String,
    pub message_id: String,
    pub current_reactions: Vec<ReactionInfo>,
}
impl AbstractEvent for MessageReactionsUpdatedEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct SubChannelMessageUpdatedEvent {
    pub guild_id: String,
    pub channel_id: String,
    //操作者
    pub user_id: String,
    // 操作者
    pub operator_id: String,
    pub old_info: SubChannelInfo,
    pub new_info: SubChannelInfo,
}
impl AbstractEvent for SubChannelMessageUpdatedEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct SubChannelCreatedEvent {
    pub guild_id: String,
    pub channel_id: String,
    //操作者
    pub user_id: String,
    // 操作者
    pub operator_id: String,
    pub channel_info: SubChannelInfo,
}
impl AbstractEvent for SubChannelCreatedEvent {}
#[derive(Deserialize, Debug, Clone)]
pub struct SubChannelDestroyedEvent {
    pub guild_id: String,
    pub channel_id: String,
    //操作者
    pub user_id: String,
    // 操作者
    pub operator_id: String,
    pub channel_info: SubChannelInfo,
}
impl AbstractEvent for SubChannelDestroyedEvent {}
