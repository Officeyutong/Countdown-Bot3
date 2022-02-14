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
        self.text.clone()
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
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ImageData {
    pub file: String,
    pub r#type: Option<String>,
    pub url: Option<String>,
    pub cache: Option<bool>,
    pub proxy: Option<bool>,
    pub timeout: Option<i64>,
    pub id: Option<String>,
}
impl_cq_tostring!(ImageData, image);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RecordData {
    pub file: String,
    pub magic: Option<i32>,
    pub url: Option<String>,
    pub cache: Option<bool>,
    pub proxy: Option<bool>,
    pub timeout: Option<i64>,
}
impl_cq_tostring!(RecordData, record);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VideoData {
    pub file: String,
    pub url: Option<String>,
    pub cache: Option<bool>,
    pub proxy: Option<bool>,
    pub timeout: Option<i64>,
}
impl_cq_tostring!(VideoData, video);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AtData {
    pub qq: String,
    pub name: Option<String>,
}
impl_cq_tostring!(AtData, at);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AnonymousData {
    pub ignore: Option<bool>,
}
impl_cq_tostring!(AnonymousData, anonymous);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ShareData {
    pub url: String,
    pub title: String,
    pub content: Option<String>,
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
    pub title: Option<String>,
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
    pub content: Option<String>,
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
    pub id: Option<i64>,
    pub text: Option<String>,
    pub qq: Option<i64>,
    pub time: Option<i64>,
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
    pub id: Option<String>,
    pub name: Option<String>,
    pub uin: Option<i64>,
    pub content: Option<Vec<Box<MessageSegment>>>,
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
    pub minwidth: Option<i64>,
    pub minheight: Option<i64>,
    pub maxwidth: Option<i64>,
    pub maxheight: Option<i64>,
    pub source: Option<String>,
    #[serde(rename = "icon")]
    pub icon_url: Option<String>,
}
impl_cq_tostring!(CardImageData, cardimage);
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TTSData {
    pub text: String,
}
impl_cq_tostring!(TTSData, tts);
