use serde::Deserialize;

pub fn create_uid() -> String {
    use rand::{distributions::Alphanumeric, Rng};

    return rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>();
}
#[derive(Deserialize, Debug)]
pub struct GeneralResponse<T> {
    pub code: String,
    pub data: T,
}

#[derive(Deserialize, Debug)]
pub struct GetKeywordResponse {
    pub ch: String,
    // pub en: String,
}
#[derive(Deserialize, Debug)]
pub struct SimpleCeleryID {
    pub celery_id: String,
    // pub code: String,
}


