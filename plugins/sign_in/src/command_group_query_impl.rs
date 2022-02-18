use crate::SignInPlugin;
use anyhow::anyhow;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use log::debug;

impl SignInPlugin {
    pub async fn command_group_query(
        &self,
        args: &Vec<String>,
        sender: &SenderType,
    ) -> ResultType<()> {
        use chrono::prelude::*;
        let config = self.config.as_ref().unwrap();
        let now = Local::now();
        let (year, month) = if args.is_empty() {
            (now.year(), now.month())
        } else if args.len() == 1 {
            (
                now.year(),
                u32::from_str_radix(&args[0], 10).map_err(|_| anyhow!("请输入合法的月份!"))?,
            )
        } else {
            (
                i32::from_str_radix(&args[1], 10).map_err(|_| anyhow!("请输入合法的年份!"))?,
                u32::from_str_radix(&args[0], 10).map_err(|_| anyhow!("请输入合法的月份!"))?,
            )
        };
        debug!("Querying year = {}, month = {}", year, month);
        let query_month_begin = Local
            .datetime_from_str(
                &format!("{}-{}-1 00:00:00", year, month),
                "%Y-%m-%d %H:%M:%S",
            )?
            .timestamp();
        let query_month_end = if month == 12 {
            Local
                .datetime_from_str(
                    &format!("{}-{}-1 00:00:00", year + 1, month),
                    "%Y-%m-%d %H:%M:%S",
                )?
                .timestamp()
                - 1
        } else {
            Local
                .datetime_from_str(
                    &format!("{}-{}-1 00:00:00", year, month + 1),
                    "%Y-%m-%d %H:%M:%S",
                )?
                .timestamp()
                - 1
        };
        debug!("Querying {} ~ {}", query_month_begin, query_month_end);
        let (user_id, group_id) = match sender {
            SenderType::Group(e) => (e.user_id as i64, e.group_id),
            _ => todo!(),
        };
        let group_sign_in_data = self
            .get_sign_in_data(query_month_begin, query_month_end, group_id, Some(user_id))
            .await?;
        let mut buf = format!(
            "[CQ:at,qq={}]\n查询到{}条签到记录：\n",
            user_id,
            group_sign_in_data.len()
        );
        let hide_score = config.hide_score_groups.contains(&group_id);
        if hide_score {
            buf.push_str("时间 日期\n");
        } else {
            buf.push_str("日期 时间 积分 积分变化\n");
        }
        for item in group_sign_in_data.iter() {
            let time_str = NaiveDateTime::from_timestamp_opt(item.time, 0)
                .map(|v| v.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or("时间格式错误".to_string());
            if hide_score {
                buf.push_str(&time_str);
                buf.push_str("\n");
            } else {
                buf.push_str(&format!(
                    "{} {} {}\n",
                    time_str, item.score, item.score_changes
                ));
            }
        }
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender_ex(sender, &buf, false)
            .await?;
        return Ok(());
    }
}
