use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    sync::Arc,
};

use clap::ArgMatches;
use countdown_bot3::countdown_bot::{
    client::{CountdownBotClient, ResultType},
    command::SenderType,
};
use log::{debug, info};
use tokio::sync::Semaphore;
use wav::{BitDepth, Header};

use crate::{
    cache::{check_from_cache, store_into_cache},
    config::MusicGenConfig,
    notes::transform_notes,
    pysynth::pysynth_b::WaveRenderer,
    utils::command_hash,
    MusicGenPlugin,
};
use anyhow::anyhow;
impl MusicGenPlugin {
    pub async fn generate_music(
        &self,
        args: ArgMatches,
        sender: &SenderType,
        using_pasteboard: bool,
    ) {
        let semaphore = self.semaphore.as_ref().unwrap().clone();
        let client = self.client.clone().unwrap();
        let config = self.config.as_ref().unwrap().clone();
        let sender = sender.clone();
        let redis_client = self.redis_client.as_ref().unwrap().clone();
        tokio::spawn(async move {
            let msg = if let Err(e) = generate_music(
                semaphore,
                &client,
                &config,
                args,
                &sender,
                using_pasteboard,
                redis_client,
            )
            .await
            {
                Some(format!("生成音乐时发生错误:\n{}", e))
            } else {
                None
            };
            if let Some(v) = msg {
                client.clone().quick_send_by_sender(&sender, &v).await.ok();
            };
        });
    }
}
pub(crate) async fn generate_music(
    semaphore: Arc<Semaphore>,
    client: &CountdownBotClient,
    config: &MusicGenConfig,
    args: ArgMatches,
    sender: &SenderType,
    using_pasteboard: bool,
    redis_client: Arc<redis::Client>,
) -> ResultType<()> {
    let start_time = std::time::Instant::now();
    let _semaphore_permit = semaphore
        .try_acquire()
        .map_err(|_| anyhow!("当前正在执行的生成音乐任务过多，请等待其他任务执行完成后再调用"))?;
    let notes = args
        .values_of("NOTES")
        .ok_or(anyhow!("请输入音符"))?
        .collect::<Vec<&str>>();
    // debug!("{:?}", notes);
    let notes_by_track = notes.split(|v| *v == "|").collect::<Vec<&[&str]>>();
    let use_number = args.is_present("numbered");
    let bpm = args
        .value_of("bpm")
        .map(|v| u32::from_str_radix(v, 10))
        .transpose()
        .map_err(|_| anyhow!("请输入合法的BPM!"))?
        .unwrap_or(config.default_bpm as u32);
    let scale = args
        .value_of("scale")
        .map(|v| v.parse())
        .transpose()
        .map_err(|_| anyhow!("请输入合法的振幅缩放!"))?
        .unwrap_or(1.0f64);
    let major = args.value_of("major").unwrap_or("C");
    let volume = {
        match args.value_of("volume") {
            Some(v) => {
                let mut output = Vec::<u32>::new();
                for s in v.split(",") {
                    output
                        .push(u32::from_str_radix(s, 10).map_err(|_| anyhow!("非法音量: {}", s))?);
                }
                Some(output)
            }
            None => None,
        }
    };
    let will_download = args.is_present("download");
    info!("Will download = {}", will_download);
    let inverse_beats = if args.is_present("inverse") {
        Some(if let Some(v) = args.value_of("beats") {
            i64::from_str_radix(v, 10).map_err(|_| anyhow!("非法beats: {}", v))?
        } else {
            4
        })
    } else {
        None
    };
    if let Some(v) = &volume {
        if v.len() != notes_by_track.len() {
            return Err(anyhow!("如果您指定音量分配占比，那么音量数必须与音轨数相同").into());
        }
    }
    let this_hash = command_hash(
        &notes,
        use_number,
        bpm,
        scale,
        major,
        &volume,
        &inverse_beats,
    );
    let mut using_cache = false;
    let mut out_bytes: Vec<u8> = vec![];
    if config.use_cache {
        if let Some(cached) = check_from_cache(redis_client.clone(), &this_hash)
            .await
            .map_err(|e| anyhow!("检验缓存时发生错误: {}", e))?
        {
            out_bytes = cached;
            using_cache = true;
        }
    }
    if using_cache {
        client
            .quick_send_by_sender(sender, "缓存命中，发送中...")
            .await?;
    }
    if !using_cache {
        let group_id = match sender {
            SenderType::Group(v) => v.group_id,
            _ => todo!(),
        };

        let note_count: usize = notes_by_track.iter().map(|x| x.len()).sum();
        {
            let this_group_note_count_limit = *config
                .group_limits
                .get(&group_id)
                .unwrap_or(&(config.max_notes as i64));
            if note_count as i64 > this_group_note_count_limit {
                return Err(anyhow!(
                    "本群音符数上限为 {} 个! 你的文件一共包含 {} 个音符",
                    this_group_note_count_limit,
                    note_count
                )
                .into());
            }
        }
        if !using_pasteboard && note_count > config.max_notes_through_message as usize {
            return Err(anyhow!("消息过长！请使用剪贴板传递参数").into());
        }
        let transformed_number = if use_number {
            let mut tracks: Vec<Vec<String>> = vec![];
            for raw_track in notes_by_track.iter() {
                tracks.push(transform_notes(raw_track, major)?);
            }
            tracks
        } else {
            notes_by_track
                .iter()
                .map(|f| f.iter().map(|s| s.to_string()).collect::<Vec<String>>())
                .collect::<Vec<Vec<String>>>()
        };
        let mut processed_tracks: Vec<Vec<(String, f64)>> = vec![];
        let mut max_len: f64 = 0.0;
        for (index, track) in transformed_number.iter().enumerate() {
            // proessed_tracks.push();
            let (data, length) = parse_track(&track[..], &inverse_beats, bpm)?;
            if length * 60.0 > config.max_length_in_seconds as f64 {
                return Err(anyhow!(
                    "音轨 {} 的长度({}s)超出了长度限制 ({}s)",
                    index + 1,
                    length * 60.0,
                    config.max_length_in_seconds
                )
                .into());
            }
            max_len = max_len.max(length);
            info!("音轨 {}: {}音符，{}分钟", index, data.len(), length);
            processed_tracks.push(data);
        }
        // info!("{:#?}", processed_tracks);
        client
            .quick_send_by_sender(
                sender,
                &format!(
                    "生成中...共计{}个音轨, {}个音符, 最长的音轨长度为{}秒",
                    processed_tracks.len(),
                    note_count,
                    (max_len * 60.0) as i32
                ),
            )
            .await?;
        let mut rendered_data = Vec::<Vec<i16>>::new();
        for (index, track) in processed_tracks.into_iter().enumerate() {
            info!("渲染音轨 {} 中.. 音符数 {}", index + 1, track.len());
            rendered_data.push(
                tokio::task::spawn_blocking(move || {
                    WaveRenderer::default()
                        .set_bpm(bpm.clone() as i32)
                        .make_wav(&track[..])
                        .map_err(|e| anyhow!("渲染音轨 {} 时发生错误:\n{}", index.clone() + 1, e))
                })
                .await??,
            );
        }
        let mut mid_val = Vec::<i32>::new();
        let mut final_output = Vec::<i16>::new();

        let max_len = rendered_data
            .iter()
            .map(|v| v.len())
            .max()
            .ok_or(anyhow!("零个音轨，玩你妈呢？"))?;
        mid_val.resize(max_len, 0);
        final_output.resize(max_len, 0);
        let volume_sum = volume
            .as_ref()
            .map(|v| v.iter().sum())
            .unwrap_or(rendered_data.len() as u32);
        info!("合并中..");
        for (i, val) in rendered_data.iter().enumerate() {
            let curr_volume = volume.as_ref().map(|v| v[i]).unwrap_or(1);
            info!("音轨 {} 输出采样点数 {}", i + 1, val.len());
            for (j, v) in val.iter().enumerate() {
                mid_val[j] += curr_volume as i32 * *v as i32;
            }
        }
        for (i, v) in mid_val.iter().enumerate() {
            final_output[i] = (*v / volume_sum as i32) as i16;
        }
        if scale != 1.0 {
            for v in final_output.iter_mut() {
                let mut s = *v as f64 * scale;
                if s > i32::MAX as f64 {
                    s = s * i16::MAX as f64 / i32::MAX as f64;
                }
                *v = s as i16;
            }
        }
        let out_bytes_loc = {
            let mut outbuf = Cursor::<Vec<u8>>::new(Vec::new());
            wav::write(
                Header {
                    channel_count: 1,
                    sampling_rate: 44100,
                    bytes_per_sample: 2,
                    bits_per_sample: 16,
                    audio_format: 1,
                    bytes_per_second: 88200,
                },
                &BitDepth::Sixteen(final_output),
                &mut outbuf,
            )
            .map_err(|e| anyhow!("生成wav时发生错误: {}", e))?;
            let mut outval = Vec::<u8>::new();
            outbuf.seek(SeekFrom::Start(0)).unwrap();
            outbuf.read_to_end(&mut outval).unwrap();
            // tokio::fs::write("out.wav", outval).await?;
            outval
        };
        let total_seconds = start_time.elapsed().as_secs();
        client
            .quick_send_by_sender(
                sender,
                &format!("生成共耗时 {} 秒，发送中..", total_seconds),
            )
            .await?;
        out_bytes = out_bytes_loc;
    };

    client
        .quick_send_by_sender(
            sender,
            &format!("[CQ:record,file=base64://{}]", base64::encode(&out_bytes)),
        )
        .await?;
    if config.use_cache || will_download {
        store_into_cache(
            redis_client.clone(),
            &this_hash,
            &out_bytes[..],
            config.cache_timeout as usize,
        )
        .await
        .map_err(|e| anyhow!("存储到Redis时发生错误: {}", e))?;
    }
    if will_download {
        client
            .quick_send_by_sender(sender, &{
                let s = format!(
                    "下载地址 ({} 秒内有效): {}",
                    config.cache_timeout,
                    config.download.template_url.replace("[hash]", &this_hash),
                );
                debug!("Download message: {}", s);
                s
            })
            .await?;
    }
    return Ok(());
    // todo!();
}
fn parse_track(
    track: &[String],
    beats: &Option<i64>,
    bpm: u32,
) -> ResultType<(Vec<(String, f64)>, f64)> {
    let mut result: Vec<(String, f64)> = vec![];
    let mut total_minutes = 0.0f64;
    for note in track.iter() {
        let (note_name, duration) = note.split_once(".").ok_or(anyhow!("非法音符: {}", note))?;
        let mut parsed_duration: f64 = (if duration.starts_with(".") {
            format!("0{}", duration)
        } else {
            duration.to_string()
        })
        .parse()
        .map_err(|_| anyhow!("非法周期: {}", duration))?;
        if let Some(v) = &beats {
            parsed_duration = (*v) as f64 / parsed_duration;
        }
        if parsed_duration.abs() < 0.1 {
            return Err(anyhow!("abs(Duration) >= 0.1").into());
        }
        total_minutes += 4.0 / parsed_duration / (bpm as f64);
        result.push((note_name.to_string(), parsed_duration));
    }
    return Ok((result, total_minutes));
}
