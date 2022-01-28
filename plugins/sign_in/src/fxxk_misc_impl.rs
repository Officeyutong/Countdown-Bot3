use countdown_bot3::countdown_bot::client::ResultType;
use rusqlite::{params, OptionalExtension};

use crate::{SignInPlugin, models::SignInData};

impl SignInPlugin {
        /*
    返回某群某人上一次的签到记录
        若不存在，返回均为参数0的初始数据
    */
    pub async fn get_last_sign_in_data(
        &self,
        group_id: i64,
        user_id: i64,
    ) -> ResultType<SignInData> {
        let db = self.database.as_ref().unwrap().lock().await;
        let query_result = db.
        query_row("SELECT * FROM SIGNINS WHERE GROUP_ID = ? AND USER_ID = ? ORDER BY TIME DESC LIMIT 1", params![group_id, user_id], 
        |r|{
            return Ok(SignInData{
                group_id:r.get(0)?,
                user_id:r.get(1)?,
                time:r.get(2)?,
                duration:r.get(3)?,
                score:r.get(4)?,
                score_changes:r.get(5)?,
            });
        }).optional();
        return Ok(query_result?.unwrap_or(SignInData::new(group_id, user_id)));
    }
}