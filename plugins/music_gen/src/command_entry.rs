use clap::{App, Arg};
use countdown_bot3::countdown_bot::{client::ResultType, command::SenderType};
use log::error;

use crate::{help::HELP_STR, luogu_fetcher::fetch_luogu_pasteboard, MusicGenPlugin};
use anyhow::anyhow;
use async_recursion::async_recursion;

impl MusicGenPlugin {
    #[async_recursion]
    pub async fn command_entry(
        &self,
        args: Vec<String>,
        sender: &SenderType,
        from_qq: bool,
    ) -> ResultType<()> {
        let config = self.config.as_ref().unwrap();
        let help_str = HELP_STR
            .replace("{DEFAULT_BPM}", config.default_bpm.to_string().as_str())
            .replace(
                "{DEFAULT_VOLUME}",
                config.default_volume.to_string().as_str(),
            );
        let parse_result = App::new("music_gen")
            .override_help(&*help_str)
            .arg(
                Arg::new("from-paste")
                    .short('p')
                    .long("from-paste")
                    .help("使用洛谷剪贴板")
                    .takes_value(true),
            )
            .arg(
                Arg::new("numbered")
                    .short('n')
                    .long("numbered")
                    .help("使用简谱"),
            )
            .arg(Arg::new("bpm").long("bpm").help("BPM数").takes_value(true))
            .arg(
                Arg::new("major")
                    .short('m')
                    .long("major")
                    .help("大调")
                    .takes_value(true),
            )
            .arg(
                Arg::new("volume")
                    .long("volume")
                    .short('v')
                    .help("音量分配")
                    .takes_value(true),
            )
            .arg(
                Arg::new("download")
                    .long("download")
                    .short('d')
                    .help("下载音乐"),
            )
            .arg(
                Arg::new("inverse")
                    .long("inverse")
                    .short('i')
                    .help("见帮助"),
            )
            .arg(
                Arg::new("beats")
                    .long("beats")
                    .short('b')
                    .help("见帮助")
                    .takes_value(true),
            )
            .arg(
                Arg::new("scale")
                    .long("scale")
                    .short('s')
                    .help("振幅缩放")
                    .takes_value(true),
            )
            .arg(
                Arg::new("NOTES")
                    .multiple_values(true)
                    .takes_value(true)
                    .help("音符"),
            )
            .setting(clap::AppSettings::NoBinaryName)
            .setting(clap::AppSettings::DisableVersionFlag)
            .color(clap::ColorChoice::Never)
            .try_get_matches_from(
                args.join(" ")
                    .split_whitespace()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>(),
            );
        match parse_result {
            Ok(parse_ret) => {
                // debug!("{:#?}", parse_ret);
                if let Some(paste_url) = parse_ret.value_of("from-paste") {
                    if !from_qq {
                        return Err(anyhow!("你递归你🐎呢?").into());
                    }
                    let inner_content = fetch_luogu_pasteboard(paste_url).await?;
                    self.command_entry(
                        inner_content.split(" ").map(|v| v.to_string()).collect(),
                        sender,
                        false,
                    )
                    .await?;
                } else {
                    // let semaphore = self.
                    self.generate_music(parse_ret, sender, !from_qq).await;
                }
            }
            Err(parse_err) => {
                // parse_err.
                error!("{}", parse_err);
                self.client
                    .as_ref()
                    .unwrap()
                    .quick_send_by_sender(&sender, &help_str)
                    .await?;
            }
        };
        return Ok(());
    }
}
