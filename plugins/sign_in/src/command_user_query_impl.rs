use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};

use crate::SignInPlugin;

impl SignInPlugin {
    pub async fn command_user_query(&self, sender: &SenderType) -> ResultType<()> {
        let user_id = match sender {
            SenderType::Console(_) => todo!(),
            SenderType::Private(evt) => evt.user_id,
            SenderType::Group(evt) => evt.user_id,
        };
        let user_data = self.get_user_data(user_id.into()).await?;
        let mut buf = format!("查询到您在{}个群有签到记录：\n", user_data.len());
        let config = self.config.as_ref().unwrap();
        let to_send = user_data
            .iter()
            .map(|r| {
                if config.hide_score_groups.contains(&r.group_id) {
                    format!("群 {} 隐藏了积分", r.group_id)
                } else {
                    format!("群 {} 积分为：{}", r.group_id, r.score)
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        buf.push_str(&to_send);
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender(sender, &buf)
            .await?;
        return Ok(());
    }
}
