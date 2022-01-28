#[derive(Debug)]
pub struct SignInData {
    pub group_id: i64,
    pub user_id: i64,
    pub time: i64,
    pub duration: i64,
    pub score: i64,
    pub score_changes: i64,
}
impl SignInData {
    pub fn new(group_id: i64, user_id: i64) -> Self {
        Self {
            group_id,
            user_id,
            duration: 0,
            score: 0,
            score_changes: 0,
            time: 0,
        }
    }
}
#[derive(Debug)]
pub struct UserData {
    pub group_id: i64,
    pub user_id: i64,
    pub score: i64,
}
// impl UserData {
//     pub fn new(group_id: i64, user_id: i64) -> Self {
//         Self {
//             group_id,
//             user_id,
//             score: 0,
//         }
//     }
// }

// pub struct RanklistItem {
//     pub group_id: i64,
//     pub user_id: i64,
//     pub score: i64,
//     pub last_time: i64,
//     pub month_times: i64,
//     pub total_times: i64,
// }
