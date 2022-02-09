use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use log::debug;

use crate::{models::SignInData, SignInPlugin};
use anyhow::anyhow;
impl SignInPlugin {
    pub async fn command_signin(&self, sender: &SenderType) -> ResultType<()> {
        let group_evt = match sender {
            SenderType::Group(evt) => evt,
            _ => todo!(),
        };
        let group_id = group_evt.group_id;
        let user_id = group_evt.user_id as i64;
        let config = self.config.as_ref().unwrap();
        let client = self.client.as_ref().unwrap();
        if config.black_list_groups.contains(&group_id) {
            return Err(anyhow!("签到功能在本群停用!").into());
        }
        let last_sign_in_data = self.get_last_sign_in_data(group_id, user_id).await?;
        use chrono::prelude::*;
        let last_time = chrono::NaiveDateTime::from_timestamp(last_sign_in_data.time, 0);
        let current_time = chrono::Local::now().naive_local();
        let signin_times = self.calc_sign_in_times(group_id, user_id).await?;
        let all_times = signin_times.total;
        let month_times = signin_times.current_month;
        debug!(
            "Last signin time: {:#?}, current time: {}",
            last_time, current_time
        );
        debug!("Last signin data: {:#?}", last_sign_in_data);
        if last_time.year() == current_time.year()
            && last_time.month() == current_time.month()
            && last_time.day() == current_time.day()
        {
            let mut buf = String::new();
            buf.push_str(&format!(
                "[CQ:at,qq={}]今天已经签过到啦！\n连续签到：{}天\n",
                user_id, last_sign_in_data.duration
            ));
            if !config.hide_score_groups.contains(&group_id) {
                buf.push_str(&format!("当前积分: {}\n", last_sign_in_data.score));
            }
            buf.push_str(&format!(
                "本月签到次数: {}\n累计群签到次数: {}",
                month_times, all_times
            ));
            client.quick_send_by_sender(sender, &buf).await?;
            return Ok(());
        }
        let mut sign_in_data = SignInData::new(group_id, user_id);
        sign_in_data.time = current_time.timestamp();
        if last_time.num_days_from_ce() + 1 == current_time.num_days_from_ce() {
            sign_in_data.duration = last_sign_in_data.duration + 1;
        } else {
            sign_in_data.duration = 1;
        }
        let mut duration_add = sign_in_data.duration - 1;
        if duration_add > 10 {
            duration_add = 10;
        }
        if sign_in_data.duration > 30 {
            sign_in_data.duration = 15;
        }
        /*
        # 连续签到加成计算：
        # 2-10天：天数-1
        # 10-30天：10
        # 大于30天：15
        */
        sign_in_data.score_changes = 10 + duration_add;
        sign_in_data.score = last_sign_in_data.score + sign_in_data.score_changes;
        self.save_data(&sign_in_data).await?;
        let mut buf = String::new();
        buf.push_str(&format!("给[CQ:at,qq={}]签到成功了！\n", user_id));
        buf.push_str(&format!("连续签到：{}天\n", sign_in_data.duration));
        if !config.hide_score_groups.contains(&group_id) {
            buf.push_str(&format!(
                "积分增加：{}\n连续签到加成：{}\n当前积分：{}\n",
                sign_in_data.score_changes, duration_add, sign_in_data.score
            ));
        }
        buf.push_str(&format!(
            "本月签到次数：{}\n累计群签到次数：{}",
            month_times, all_times
        ));

        client.quick_send_by_sender(sender, &buf).await?;
        return Ok(());
    }
}
