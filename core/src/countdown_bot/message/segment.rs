use crate::impl_cq_tostring;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone)]
pub enum MessageSegment {
    Text(TextData),
    Face(FaceData),
    Record(RecordData),
    Video(VideoData),
    At(AtData),
    RPS,
    Dice,
    Shake,
    Anonymous,
    Share(ShareData),
    Contact(ContactData),
    Location(LocationData),
    Music(MusicData),
    Image(ImageData),
    Reply(ReplyData),
    Redbag(RedbagData),
    Poke(PokeData),
    Gift(GiftData),
    Forward(ForwardData),
    Node(NodeData),
    XML(XMLData),
    JSON(JSONData),
    CardImage(CardImageData),
    TTS(TTSData),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TextData {
    pub text: String,
}
impl ToString for TextData {
    fn to_string(&self) -> String {
        html_escape::encode_unquoted_attribute(&self.text).to_string()
    }
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FaceData {
    pub id: i64,
}
impl ToString for FaceData {
    fn to_string(&self) -> String {
        format!("[CQ:face,id={}]", self.id)
    }
}
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct ImageData {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
impl_cq_tostring!(ImageData, image);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RecordData {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magic: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
}
impl_cq_tostring!(RecordData, record);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VideoData {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
}
impl_cq_tostring!(VideoData, video);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AtData {
    pub qq: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
impl_cq_tostring!(AtData, at);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AnonymousData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<bool>,
}
impl_cq_tostring!(AnonymousData, anonymous);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ShareData {
    pub url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}
impl_cq_tostring!(ShareData, share);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ContactData {
    pub id: i64,
    pub r#type: ContactType,
}
impl_cq_tostring!(ContactData, contact);
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ContactType {
    Qq,
    Group,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LocationData {
    pub lat: String,
    pub lon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}
impl_cq_tostring!(LocationData, location);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MusicData {
    pub r#type: MusicType,
    pub id: String,
    pub url: String,
    pub audio: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}
impl_cq_tostring!(MusicData, music);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum MusicType {
    #[serde(rename = "qq")]
    TypeQQ,
    #[serde(rename = "163")]
    Type163,
    #[serde(rename = "xm")]
    TypeXM,
    #[serde(rename = "custom")]
    TypeCustom,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ReplyData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qq: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<i64>,
}
impl_cq_tostring!(ReplyData, reply);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RedbagData {
    pub title: String,
}
impl_cq_tostring!(RedbagData, redbag);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PokeData {
    pub qq: i64,
}
impl_cq_tostring!(PokeData, poke);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GiftData {
    pub qq: i64,
    pub id: i64,
}
impl_cq_tostring!(GiftData, gift);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ForwardData {
    pub id: String,
}
impl_cq_tostring!(ForwardData, forward);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NodeData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uin: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<Box<MessageSegment>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<Vec<Box<MessageSegment>>>,
}
impl_cq_tostring!(NodeData, node);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct XMLData {
    pub data: String,
}
impl_cq_tostring!(XMLData, xml);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct JSONData {
    pub data: String,
}
impl_cq_tostring!(JSONData, json);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CardImageData {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minwidth: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minheight: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxwidth: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxheight: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(rename = "icon", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}
impl_cq_tostring!(CardImageData, cardimage);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TTSData {
    pub text: String,
}
impl_cq_tostring!(TTSData, tts);
