use serde::Deserialize;
use serde_json::Value;

use crate::{
    countdown_bot::message::{segment::MessageSegment, wrapper::Message},
    declare_api_call,
};

use super::CountdownBotClient;
#[derive(Deserialize, Debug, Clone)]
pub struct GetImageResponse {
    pub size: i64,
    pub filename: String,
    pub url: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GetMessageResponse {
    pub message_id: i64,
    pub real_id: i64,
    pub sender: Value,
    pub time: i64,
    pub message: Message,
    pub raw_message: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct ForwardMessageSender {
    pub nickname: String,
    pub user_id: i64,
}
#[derive(Deserialize, Debug, Clone)]
pub struct ForwardMessageEntry {
    pub content: Message,
    pub time: i64,
    pub sender: ForwardMessageSender,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GetForwardMessageResponse {
    pub messages: Vec<ForwardMessageEntry>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct InvitedRequest {
    pub request_id: i64,
    pub invitor_uin: i64,
    pub invitor_nick: String,
    pub group_id: i64,
    pub group_name: String,
    pub checked: bool,
    pub actor: i64,
}
#[derive(Deserialize, Debug, Clone)]
pub struct JoinRequest {
    pub request_id: i64,
    pub requester_uin: i64,
    pub requester_nick: String,
    pub message: String,
    pub group_id: i64,
    pub group_name: String,
    pub checked: bool,
    pub actor: i64,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GetGroupSystemMessageResponse {
    pub invited_requests: Vec<InvitedRequest>,
    pub join_requests: Vec<JoinRequest>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GetGroupFileSystemInfoResponse {
    pub file_count: i64,
    pub limit_count: i64,
    pub used_space: i64,
    pub total_space: i64,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupFile {
    pub group_id: i64,
    pub file_id: String,
    pub file_name: String,
    pub busid: i32,
    pub file_size: i64,
    pub upload_time: i64,
    pub dead_time: i64,
    pub modify_time: i64,
    pub download_times: i64,
    pub uploader: i64,
    pub uploader_name: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GroupFolder {
    pub group_id: i64,
    pub folder_id: String,
    pub folder_name: String,
    pub create_time: i64,
    pub creator: i64,
    pub creator_name: String,
    pub total_file_count: i64,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GetGroupFilesResponse {
    pub files: Vec<GroupFile>,
    pub folders: Vec<GroupFolder>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GetGroupFileUrlResponse {
    pub url: String,
}
impl CountdownBotClient {
    declare_api_call!(
        set_group_portrait,
        (),
        (group_id, i64),
        (file, &str),
        (cache, i32)
    );
    declare_api_call!(get_image, GetImageResponse, (file, &str));
    declare_api_call!(get_msg, GetMessageResponse, (message_id, i64));
    declare_api_call!(get_forward_msg, GetForwardMessageResponse, (mesage_id, i64));
    declare_api_call!(
        send_group_forward_msg,
        (),
        (group_id, i64),
        (messages, Vec<MessageSegment>)
    );
    declare_api_call!(get_group_system_msg, Option<GetGroupSystemMessageResponse>,);
    declare_api_call!(
        get_group_file_system_info,
        GetGroupFileSystemInfoResponse,
        (group_id, i64)
    );
    declare_api_call!(get_group_root_files, GetGroupFilesResponse, (group_id, i64));
    declare_api_call!(
        get_group_files_by_folder,
        GetGroupFilesResponse,
        (group_id, i64),
        (folder_id, &str)
    );
    declare_api_call!(
        get_group_file_url,
        GetGroupFileUrlResponse,
        (group_id, i64),
        (file_id, &str),
        (busid, i32)
    );
}
