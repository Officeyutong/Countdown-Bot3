use crate::CatsPlugin;
use anyhow::anyhow;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use rusqlite::params;

impl CatsPlugin {
    pub async fn delete_cat(&self, sender: &SenderType, id: &str) -> ResultType<()> {
        let conn = self.database.as_ref().unwrap().lock().await;
        let id = i64::from_str_radix(id, 10).map_err(|_| anyhow!("请输入合法整数!"))?;
        {
            let mut stmt = conn.prepare("SELECT USER_ID FROM CATS WHERE ID = ?")?;
            let upload_uid: i64 = match stmt.query_row(params![id], |r| r.get(0)) {
                Ok(v) => v,
                Err(e) => match e {
                    rusqlite::Error::QueryReturnedNoRows => {
                        return Err(anyhow!("非法猫片ID: {}", id).into())
                    }
                    err => return Err(err.into()),
                },
            };
            let can_delete = match sender {
                SenderType::Console(_) => true,
                SenderType::Private(e) => e.user_id as i64 == upload_uid,
                SenderType::Group(e) => e.user_id as i64 == upload_uid,
                SenderType::Guild(_) => return Err(anyhow!("暂不支持在频道内操作!").into()),
            };
            if !can_delete {
                return Err(anyhow!("你只能删除你自己上传的猫片!").into());
            }
            conn.execute("DELETE FROM CATS WHERE ID = ?", params![id])?;
        }
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender(sender, "删除成功!")
            .await?;
        return Ok(());
    }
    pub async fn list_cat(&self, sender: &SenderType, qq: Option<String>) -> ResultType<()> {
        let conn = self.database.as_ref().unwrap().lock().await;
        let mut buf = String::new();
        use fallible_iterator::FallibleIterator;
        if let Some(qq) = qq {
            buf.push_str(format!("用户 {} 上传的猫片:\n", qq).as_str());
            let mut stmt = conn.prepare("SELECT ID FROM CATS WHERE USER_ID = ?")?;
            let ids: Vec<i64> = stmt
                .query(params![
                    i64::from_str_radix(&qq, 10).map_err(|_| anyhow!("非法QQ号: {}", qq))?
                ])?
                .map(|r| r.get(0))
                .collect()
                .map_err(|e| anyhow!("执行SQL时发生错误: {}", e))?;
            ids.iter()
                .for_each(|x| buf.push_str(format!("{}\n", x).as_str()));
        } else {
            buf.push_str("上传过猫片的用户:\n");
            let mut stmt = conn.prepare("SELECT DISTINCT USER_ID FROM CATS")?;

            let ids: Vec<i64> = stmt
                .query(params![])?
                .map(|r| r.get(0))
                .collect()
                .map_err(|e| anyhow!("执行SQL时发生错误: {}", e))?;
            ids.iter()
                .for_each(|x| buf.push_str(format!("{}\n", x).as_str()));
        }
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender(sender, buf.as_str())
            .await?;
        return Ok(());
    }
}
