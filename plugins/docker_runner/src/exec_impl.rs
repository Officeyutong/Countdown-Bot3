use crate::DockerRunnerPlugin;
use anyhow::anyhow;
use bollard::{
    container::{Config, LogOutput, LogsOptions},
    models::{HostConfig, Mount, MountTypeEnum},
};
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use futures_util::{sink, stream::StreamExt};
use log::info;
use std::time::Duration;
const APP_NAME: &str = "app";
impl DockerRunnerPlugin {
    pub async fn handle_exec(
        &mut self,
        sender: &SenderType,
        code: &str,
        language: &str,
    ) -> ResultType<()> {
        let config = self.config.clone().unwrap();
        let lang_config = config
            .language_setting
            .get(language)
            .ok_or(anyhow!("非法语言ID: {}", language))?;
        let input_data = self.get_cache(sender).await.unwrap_or("".to_string());
        let sender_evt = match sender {
            SenderType::Group(e) => e,
            _ => todo!(),
        };
        let uid = sender_evt.user_id;
        if config.blacklist_users.contains(&(uid as i64)) {
            return Err(anyhow!("你不被允许使用该指令!").into());
        }
        info!("Code = \n{}", code);
        let docker_client = bollard::Docker::connect_with_socket_defaults()
            .map_err(|e| anyhow!("初始化Docker时发生错误: {}", e))?;
        let working_dir = tempfile::tempdir()?;
        let source_filename = lang_config.source_file(APP_NAME);
        let exec_filename = lang_config.executable_file(APP_NAME);
        tokio::fs::write(working_dir.path().join(&source_filename), code).await?;
        tokio::fs::write(working_dir.path().join("f_stdin"), input_data).await?;
        let command = format!(
            "{} && {} < f_stdin",
            lang_config.compile_arg(&source_filename, &exec_filename),
            lang_config.run_arg(&exec_filename)
        );
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
                        memory: Some(50 * ((1 << 20) as i64)),
                        memory_swap: Some(50 * ((1 << 20) as i64)),
                        oom_kill_disable: Some(false),
                        nano_cpus: Some((0.4 / 1e-9) as i64),
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
            Duration::from_millis(config.execute_time_limit as u64),
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
            self.client
                .as_ref()
                .unwrap()
                .quick_send_by_sender(sender, "执行超时!")
                .await?;
        }
        let output = docker_client
            .logs(
                &container.id,
                Some(LogsOptions::<&str> {
                    stderr: true,
                    stdout: true,
                    follow: true,
                    ..Default::default()
                }),
            )
            .collect::<Vec<Result<LogOutput, bollard::errors::Error>>>()
            .await;
        let client = self.client.as_ref().unwrap();
        if output.is_empty() {
            client.quick_send_by_sender(sender, "无输出!").await?;
        } else {
            let mut lines = vec![];
            let mut line_overflow = false;
            let mut length_overflow = false;
            for line in output.iter() {
                if let Ok(v) = line {
                    match v {
                        LogOutput::Console { message } => {
                            lines.push(message);
                            if lines.len() >= config.new_line_count_limit as usize {
                                line_overflow = true;
                                break;
                            }
                        }
                        _ => continue,
                    };
                }
            }
            let mut output = String::new();
            for line in lines {
                output.push_str(&String::from_utf8(
                    line.iter().map(|x| *x).collect::<Vec<u8>>(),
                )?);
                if output.len() > config.outout_length_limit as usize {
                    length_overflow = true;
                    break;
                }
            }
            if length_overflow {
                output = output
                    .chars()
                    .take(config.outout_length_limit as usize)
                    .collect::<String>();
                output.push_str("\n[超出长度部分已截断]");
            }
            if line_overflow {
                output.push_str("\n[超出行数已截断]");
            }
            client.quick_send_by_sender(sender, &output).await?;
        }
        return Ok(());
    }
}
