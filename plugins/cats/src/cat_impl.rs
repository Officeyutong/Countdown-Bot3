use crate::{models::CatImage, CatsPlugin};
use anyhow::anyhow;
use clap::ArgMatches;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use rusqlite::{params, OptionalExtension};

impl CatsPlugin {
    pub async fn cat_command(&self, sender: &SenderType, arg: ArgMatches<'_>) -> ResultType<()> {
        let conn = self.database.as_ref().unwrap().lock().await;
        if !conn.prepare("SELECT COUNT(*) FROM CATS")?.exists([])? {
            return Err(anyhow!("当前无人上传过猫片!").into());
        }
        let qq = arg
            .value_of("qq")
            .map(|e| i64::from_str_radix(e, 10))
            .transpose()
            .map_err(|_| anyhow!("请输入合法QQ号!"))?;
        let id = arg
            .value_of("id")
            .map(|e| i64::from_str_radix(e, 10))
            .transpose()
            .map_err(|_| anyhow!("请输入合法ID!"))?;
        if qq.is_some() && id.is_some() {
            return Err(anyhow!("不能同时指定ID和QQ!").into());
        }
        let to_send_id = if let Some(qq) = qq {
            conn.query_row(
                "SELECT ID FROM CATS WHERE USER_ID = ? ORDER BY RANDOM() LIMIT 1",
                params![qq],
                |r| r.get(0),
            )
            .optional()?
            .ok_or(anyhow!("该用户没有上传过猫片!"))?
        } else if let Some(id) = id {
            id
        } else {
            conn.query_row(
                "SELECT ID FROM CATS ORDER BY RANDOM() LIMIT 1",
                [],
                |r| r.get(0),
            )?
        };

        let data = conn
            .query_row(
                "SELECT ID,USER_ID,UPLOAD_TIME,DATA FROM CATS WHERE ID = ?",
                params![to_send_id],
                |r| {
                    Ok(CatImage {
                        checksum: String::new(),
                        data: r.get(3)?,
                        id: r.get(0)?,
                        upload_time: r.get(2)?,
                        user_id: r.get(1)?,
                    })
                },
            )
            .optional()?
            .ok_or(anyhow!("非法猫片ID: {}", to_send_id))?;
        let upload_time_str = if data.upload_time == 0 {
            None
        } else {
            use chrono::prelude::*;
            let upload_time = DateTime::<Local>::from_utc(
                NaiveDateTime::from_timestamp(data.upload_time, 0),
                *Local.timestamp(0, 0).offset(),
            );
            Some(format!(
                "(上传于 {}年{}月{}日)",
                upload_time.year(),
                upload_time.month(),
                upload_time.day()
            ))
        };
        let b64enc = base64::encode(data.data.ok_or(anyhow!("此猫片不存在数据!"))?);
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender(
                sender,
                format!(
                    "来自 {} 的猫片 ID:{} {}\n[CQ:image,file=base64://{}]",
                    data.user_id,
                    data.id,
                    upload_time_str.unwrap_or(String::new()),
                    b64enc
                )
                .as_str(),
            )
            .await?;
        return Ok(());
    }
}
