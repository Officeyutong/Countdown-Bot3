use std::time::Duration;

use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use log::{error, info};

use crate::{sam::SAMPool, DSDrawerPlugin};
use anyhow::anyhow;
impl DSDrawerPlugin {
    pub async fn generate_sam(&self, text: &str, sender: &SenderType) -> ResultType<()> {
        info!("Generating SAM: {}", text);
        let config = self.config.as_ref().unwrap();
        let text_cloned = text.to_string();
        let dot_data = tokio::task::spawn_blocking(move || {
            let mut pool = SAMPool::default();
            for (id, s) in text_cloned.split("|").enumerate() {
                pool.join_string(s, id as i32);
            }
            pool.collect();
            pool.generate_graph()
        })
        .await?;
        let work_dir = tempfile::tempdir()?;
        let dot_file = work_dir.path().join("out.dot");
        let png_file = work_dir.path().join("out.png");
        tokio::fs::write(dot_file.clone(), dot_data).await?;
        let mut process = tokio::process::Command::new(config.dot_executable.clone())
            .arg("-Tpng")
            .args(&["-o", png_file.to_str().unwrap()])
            .arg(dot_file.to_str().unwrap())
            .spawn()?;
        info!("Rendering image to {:?}, using {:?}", png_file, dot_file);
        match tokio::time::timeout(
            Duration::from_secs(config.dot_timeout as u64),
            process.wait(),
        )
        .await
        {
            Err(e) => {
                process.kill().await?;
                return Err(anyhow!("dot执行超时!\n{}", e).into());
            }
            Ok(ret) => {
                let exit_status = ret?;
                if !exit_status.success() {
                    error!("{}", exit_status);
                    return Err(anyhow!("执行dot失败!\n{}", exit_status).into());
                } else {
                    let img_data = tokio::fs::read(png_file).await?;
                    let b64enc = base64::encode(img_data);
                    self.client
                        .clone()
                        .unwrap()
                        .quick_send_by_sender_ex(
                            sender,
                            format!("[CQ:image,file=base64://{}]", b64enc).as_str(),
                            false,
                        )
                        .await?;
                    return Ok(());
                }
            }
        };
    }
}
