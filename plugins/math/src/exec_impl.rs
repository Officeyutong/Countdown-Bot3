use anyhow::anyhow;
use bollard::{
    container::Config,
    models::{HostConfig, Mount, MountTypeEnum},
};
use countdown_bot3::countdown_bot::{
    client::{CountdownBotClient, ResultType},
    command::SenderType,
    message::{
        segment::{ImageData, MessageSegment, TextData},
        wrapper::Message,
    },
};
use futures_util::{sink, stream::StreamExt};
use log::info;
use serde::Deserialize;
use std::time::Duration;

use crate::MathPlugin;
const SRC_NAME: &str = "run.py";
#[derive(Debug)]
pub struct ExecuteResult {
    pub image: String,
    pub latex: String,
    pub python_expr: String,
    pub error: String,
}
#[derive(Deserialize)]
pub struct LocalResult {
    pub image: String,
    pub latex: String,
    pub python_expr: String,
}
impl ExecuteResult {
    pub async fn send_to(
        &self,
        sender: &SenderType,
        client: &CountdownBotClient,
    ) -> ResultType<()> {
        client
            .msgseg_quicksend(
                sender,
                &Message::Segment(vec![
                    MessageSegment::Text(TextData {
                        text: format!(
                            r###"Python表达式:
{}
    
LaTeX:
{}
    
图像:
"###,
                            self.python_expr, self.latex
                        ),
                    }),
                    MessageSegment::Image(ImageData {
                        file: format!("base64://{}", self.image),
                        ..Default::default()
                    }),
                ]),
            )
            .await?;
        return Ok(());
    }
}

impl MathPlugin {
    pub async fn handle_exec(
        &self,
        code: &str,
        custom_timeout_message: Option<&str>,
    ) -> ResultType<ExecuteResult> {
        let config = self.config.clone().unwrap();

        info!("Code = \n{}", code);
        let docker_client = bollard::Docker::connect_with_socket_defaults()
            .map_err(|e| anyhow!("初始化Docker时发生错误: {}", e))?;
        let working_dir = tempfile::tempdir()?;
        tokio::fs::write(working_dir.path().join(&SRC_NAME), code).await?;
        let command = format!("python -O {} 2> err.txt", SRC_NAME);
        info!("Working directory: {:?}", working_dir.path());
        info!("Command line: {}", command);
        let container = docker_client
            .create_container::<String, String>(
                None,
                Config {
                    image: Some(config.docker_image),
                    cmd: Some(vec!["sh".to_string(), "-c".to_string(), command]),
                    open_stdin: Some(true),
                    // detach
                    tty: Some(true),
                    network_disabled: Some(true),
                    working_dir: Some("/temp".to_string()),
                    host_config: Some(HostConfig {
                        mounts: Some(vec![Mount {
                            target: Some("/temp".to_string()),
                            source: Some(working_dir.path().to_str().unwrap().to_string()),
                            read_only: Some(false),
                            typ: Some(MountTypeEnum::BIND),
                            ..Default::default()
                        }]),
                        memory: Some(128 * ((1 << 20) as i64)),
                        memory_swap: Some(128 * ((1 << 20) as i64)),
                        oom_kill_disable: Some(false),
                        nano_cpus: Some((0.4 / 1e-9) as i64),
                        auto_remove: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;
        let local_client = docker_client.clone();
        let container_id = container.id.clone();
        docker_client
            .start_container::<&str>(&container.id, None)
            .await
            .map_err(|e| anyhow!("启动容器时发生错误: {}", e))?;
        if let Err(_) = tokio::time::timeout(
            Duration::from_millis(config.default_timeout as u64),
            async move {
                let client = local_client;
                client
                    .wait_container::<&str>(&container_id, None)
                    .map(|x| Ok(x.unwrap()))
                    .forward(sink::drain())
                    .await
                    .ok();
            },
        )
        .await
        {
            docker_client
                .kill_container::<&str>(&container.id, None)
                .await
                .ok();
            docker_client.stop_container(&container.id, None).await.ok();
            return Err(anyhow!("{}", custom_timeout_message.unwrap_or("执行超时!")).into());
        }
        // Box::leak(Box::from(working_dir));
        let error_text = tokio::fs::read_to_string(working_dir.path().join("err.txt"))
            .await
            .ok();
        let output = tokio::fs::read_to_string(working_dir.path().join("output.txt"))
            .await
            .map_err(|e| {
                anyhow!(
                    "读取结果文件时发生错误:\n{}\n程序标准错误:\n{}",
                    e,
                    error_text.as_ref().unwrap_or(&"".to_string())
                )
            })?;

        let local_result = serde_json::from_str::<LocalResult>(output.as_str())?;
        return Ok(ExecuteResult {
            error: error_text.unwrap_or(String::new()),
            image: local_result.image,
            latex: local_result.latex,
            python_expr: local_result.python_expr,
        });
    }
}
