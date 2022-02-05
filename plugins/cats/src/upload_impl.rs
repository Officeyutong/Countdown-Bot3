use crate::CatsPlugin;
use anyhow::anyhow;
use clap::ArgMatches;
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use rusqlite::{params, OptionalExtension};
impl CatsPlugin {
    pub async fn upload_command(
        &self,
        sender: &SenderType,
        args: ArgMatches<'_>,
    ) -> ResultType<()> {
        lazy_static! {
            static ref EXPR_IMAGE: Regex =
                Regex::new(r"\[CQ:image.+url=(?P<url>[^\[^\]]+)").unwrap();
        }
        let config = self.config.as_ref().unwrap();
        let as_qq = args
            .value_of("as-qq")
            .map(|v| i64::from_str_radix(v, 10))
            .transpose()
            .map_err(|_| anyhow!("请输入合法的QQ号!"))?;
        let image_data = args
            .value_of("IMAGE")
            .ok_or(anyhow!("请把图片随指令一起发送!"))?;
        debug!("image: {}", image_data);
        let captures = EXPR_IMAGE
            .captures(image_data)
            .ok_or(anyhow!("没有在上传的图片中找到URL!"))?;
        let url = captures.name("url").unwrap().as_str();
        let image_bytes = reqwest::get(url)
            .await
            .map_err(|e| anyhow!("下载图片时发生错误: {}", e))?
            .bytes()
            .await?
            .to_vec();
        if image_bytes.len() > config.image_size_limit.try_into().unwrap() {
            return Err(anyhow!("图片过大！").into());
        }
        let uploader_id = match sender {
            SenderType::Console(_) => todo!(),
            SenderType::Private(evt) => evt.user_id,
            SenderType::Group(evt) => evt.user_id,
        };
        let in_whitelist = config.white_list_users.contains(&(uploader_id as i64));
        if !config.white_list_users.contains(&(uploader_id as i64)) && as_qq.is_some() {
            return Err(anyhow!("只有白名单用户才可以以其他人的身份上传图片").into());
        }
        let should_check = config.ensure_can_upload(sender)?;
        if should_check {
            // 尝试时间间隔
            let mut last_try_map = self.last_try.as_ref().lock().await;
            let now = chrono::Local::now().timestamp();
            if now
                - last_try_map
                    .get(&(uploader_id as i64))
                    .map(|x| *x)
                    .unwrap_or(0)
                < config.try_delay
            {
                return Err(anyhow!(
                    "您不在白名单之中，为了防止滥用识别资源，您在 {} 秒内只能尝试上传一次。",
                    config.try_delay
                )
                .into());
            }
            let message = self.recognize_cat_image(&image_bytes).await?;
            self.client
                .as_ref()
                .unwrap()
                .quick_send_by_sender(sender, &message)
                .await?;
            last_try_map.insert(uploader_id.into(), now);
        }
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender(sender, "开始保存猫猫图片...")
            .await?;
        let conn = self.database.as_ref().unwrap().lock().await;
        let last_upload: i64 = conn
            .query_row(
                "SELECT UPLOAD_TIME FROM CATS WHERE USER_ID = ? ORDER BY UPLOAD_TIME DESC",
                params![uploader_id],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0);
        if !in_whitelist && (chrono::Local::now().timestamp() - last_upload < config.success_delay)
        {
            return Err(anyhow!(
                "您不在白名单中，为了防止滥用，您在 {} 秒内只能成功上传一次",
                config.success_delay
            )
            .into());
        }
        let image_hash = hex::encode(md5::compute(&image_bytes[..]).to_vec());
        if conn
            .prepare("SELECT CHECKSUM FROM CATS WHERE CHECKSUM = ?")?
            .exists(params![image_hash])?
        {
            return Err(anyhow!("之前有人上传过一样的猫片！").into());
        }
        conn.execute(
            "INSERT INTO CATS (USER_ID,UPLOAD_TIME,DATA,CHECKSUM) VALUES (?,?,?,?)",
            params![
                // 考虑以其他人的身份上传
                as_qq.unwrap_or(uploader_id),
                chrono::Local::now().timestamp(),
                image_bytes,
                image_hash
            ],
        )?;
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender(
                sender,
                &format!(
                    "用户 {} 的猫猫图片 {} 上传成功!",
                    uploader_id,
                    conn.last_insert_rowid()
                ),
            )
            .await?;
        return Ok(());
    }
}
