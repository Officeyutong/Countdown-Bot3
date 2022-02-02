use crate::{
    models::{SignInData, UserData},
    SignInPlugin,
};
use chrono::{Datelike, TimeZone};
use countdown_bot3::countdown_bot::client::ResultType;
use fallible_iterator::FallibleIterator;
use rusqlite::{params, OptionalExtension};
pub struct SigninCount {
    pub total: i64,
    pub current_month: i64,
}
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
        let query_result = db
            .query_row(
                "SELECT * FROM SIGNINS \
         WHERE GROUP_ID = ? AND USER_ID = ? \
          ORDER BY TIME DESC LIMIT 1",
                params![group_id, user_id],
                |r| {
                    return Ok(SignInData {
                        group_id: r.get(0)?,
                        user_id: r.get(1)?,
                        time: r.get(2)?,
                        duration: r.get(3)?,
                        score: r.get(4)?,
                        score_changes: r.get(5)?,
                    });
                },
            )
            .optional();
        return Ok(query_result?.unwrap_or(SignInData::new(group_id, user_id)));
    }
    // pub async fn get_last_sign_in_time(
    //     &self,
    //     group_id: i64,
    //     user_id: i64,
    // ) -> ResultType<Option<i64>> {
    //     let db = self.database.as_ref().unwrap().lock().await;
    //     return Ok(db
    //         .query_row(
    //             "SELECT MAX(TIME) FROM SIGNINS WHERE GROUP_ID = ? AND USER_ID = ?",
    //             params![group_id, user_id],
    //             |f| f.get(0),
    //         )
    //         .optional()?);
    // }
    /*
    返回两个int值，表示某群某人总签到次数和当前月份签到次数
    */
    pub async fn calc_sign_in_times(&self, group_id: i64, user_id: i64) -> ResultType<SigninCount> {
        let db = self.database.as_ref().unwrap().lock().await;
        let now = chrono::Local::now();
        let this_month_timestamp = chrono::Local
            .ymd(now.year(), now.month(), 1)
            .and_hms(0, 0, 0)
            .timestamp();
        let total_times = db
            .query_row(
                "SELECT COUNT(*) FROM SIGNINS WHERE GROUP_ID = ? AND USER_ID = ?",
                params![group_id, user_id],
                |r| r.get(0),
            )
            .optional()?;
        let month_times = db
            .query_row(
                "SELECT COUNT(*) FROM SIGNINS WHERE GROUP_ID = ? AND USER_ID = ? AND TIME >= ? ",
                params![group_id, user_id, this_month_timestamp],
                |r| r.get(0),
            )
            .optional()?;

        return Ok(SigninCount {
            current_month: month_times.unwrap(),
            total: total_times.unwrap(),
        });
    }
    //返回某人在各群的签到数据
    pub async fn get_user_data(&self, user_id: i64) -> ResultType<Vec<UserData>> {
        let db = self.database.as_ref().unwrap().lock().await;
        let mut stmt =
            db.prepare("SELECT GROUP_ID, USER_ID, SCORE FROM USERS WHERE USER_ID = ?")?;
        let result = stmt
            .query(params![user_id])?
            .map(|r| {
                Ok(UserData {
                    group_id: r.get(0)?,
                    user_id: r.get(1)?,
                    score: r.get(2)?,
                })
            })
            .collect::<Vec<UserData>>()?;

        return Ok(result);
    }
    // pub async fn get_group_sign_in_ranklist(&self, group_id: i64) -> ResultType<Vec<RanklistItem>> {
    //     let db = self.database.as_ref().unwrap().lock().await;
    //     let mut stmt =
    //         db.prepare("SELECT GROUP_ID, USER_ID, SCORE, MAX(SIGNINS.TIME) FROM USERS WHERE GROUP_ID = ? JOIN SIGNINS ON SIGNINS.GROUP_ID = USERS.GROUP_ID AND SIGNINS.USER_ID = USERS.USER_ID")?;
    //     let mut result = stmt
    //         .query(params![group_id])?
    //         .map(|r| {
    //             Ok(RanklistItem {
    //                 group_id: r.get(0)?,
    //                 user_id: r.get(1)?,
    //                 score: r.get(2)?,
    //                 last_time: r.get(3)?,
    //                 month_times: -1,
    //                 total_times: -1,
    //             })
    //         })
    //         .collect::<Vec<RanklistItem>>()?;
    //     for item in result.iter_mut() {
    //         let signin_times = self.calc_sign_in_times(item.group_id, item.user_id).await?;
    //         item.total_times = signin_times.total;
    //         item.month_times = signin_times.current_month;
    //     }
    //     result.sort_by_key(|r| -r.score);
    //     return Ok(result);
    // }
    //返回某个时间段内某群(某人)的签到记录
    pub async fn get_sign_in_data(
        &self,
        time_begin: i64,
        time_end: i64,
        group_id: i64,
        user_id: Option<i64>,
    ) -> ResultType<Vec<SignInData>> {
        let db = self.database.as_ref().unwrap().lock().await;

        let vec = if let Some(user_id) = user_id {
            let mut stmt =
            db.prepare("SELECT TIME, DURATION, SCORE, SCORE_CHANGES FROM SIGNINS WHERE GROUP_ID = ? AND USER_ID = ? AND TIME >= ? AND TIME <= ?")?;
            let x = stmt
                .query(params![group_id, user_id, time_begin, time_end])?
                .map(|r| {
                    Ok(SignInData {
                        group_id,
                        user_id,
                        time: r.get(0)?,
                        duration: r.get(1)?,
                        score: r.get(2)?,
                        score_changes: r.get(3)?,
                    })
                })
                .collect::<Vec<SignInData>>()?;
            x
        } else {
            let mut stmt =
            db.prepare("SELECT TIME, DURATION, SCORE, SCORE_CHANGES, USER_ID FROM SIGNINS WHERE GROUP_ID = ? AND TIME >= ? AND TIME <= ?")?;
            let x = stmt
                .query(params![group_id, time_begin, time_end])?
                .map(|r| {
                    Ok(SignInData {
                        group_id,
                        user_id: r.get(4)?,
                        time: r.get(0)?,
                        duration: r.get(1)?,
                        score: r.get(2)?,
                        score_changes: r.get(3)?,
                    })
                })
                .collect::<Vec<SignInData>>()?;
            x
        };

        return Ok(vec);
    }
    pub async fn save_data(&self, sign_in_data: &SignInData) -> ResultType<()> {
        let db = self.database.as_ref().unwrap().lock().await;
        db.execute("INSERT INTO SIGNINS (GROUP_ID,USER_ID,TIME,DURATION,SCORE,SCORE_CHANGES) VALUES (?,?,?,?,?,?)", params![
            sign_in_data.group_id,
            sign_in_data.user_id,
            sign_in_data.time,
            sign_in_data.duration,
            sign_in_data.score,
            sign_in_data.score_changes
        ])?;
        let exists = {
            let mut stmt = db.prepare("SELECT * FROM USERS WHERE GROUP_ID = ? AND USER_ID = ?")?;
            stmt.exists(params![sign_in_data.group_id, sign_in_data.user_id])?
        };
        if exists {
            db.execute(
                "UPDATE USERS SET SCORE = ? WHERE GROUP_ID = ? AND USER_ID = ?",
                params![
                    sign_in_data.score,
                    sign_in_data.group_id,
                    sign_in_data.user_id
                ],
            )?;
        } else {
            db.execute(
                "INSERT INTO USERS (GROUP_ID,USER_ID,SCORE) VALUES (?,?,?)",
                params![
                    sign_in_data.group_id,
                    sign_in_data.user_id,
                    sign_in_data.score
                ],
            )?;
        }
        return Ok(());
    }
}
