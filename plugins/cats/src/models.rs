#[derive(Debug)]
pub struct CatImage {
    pub id: i64,
    pub user_id: i64,
    pub upload_time: i64,
    pub data: Option<Vec<u8>>,
    pub checksum: String,
}
