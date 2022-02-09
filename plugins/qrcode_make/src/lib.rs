use anyhow::anyhow;
use async_trait::async_trait;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::CountdownBotClient,
        command::{Command, SenderType},
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    export_static_plugin,
};
use serde::{Deserialize, Serialize};
static PLUGIN_NAME: &str = "qrcode_make";

#[derive(Deserialize, Serialize)]
pub struct QRCodeConfig {
    pub max_string_length: u32,
}

impl Default for QRCodeConfig {
    fn default() -> Self {
        Self {
            max_string_length: 500,
        }
    }
}

#[derive(Default)]
struct QRCodePlugin {
    client: Option<CountdownBotClient>,
    config: Option<QRCodeConfig>,
}

#[async_trait]
impl BotPlugin for QRCodePlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        bot.register_command(
            Command::new("qrcode")
                .group(true)
                .private(true)
                .description("生成二维码 | qrcode <数据>"),
        )?;
        Ok(())
    }
    fn on_before_start(
        &mut self,
        _bot: &mut bot::CountdownBot,
        client: CountdownBotClient,
    ) -> HookResult<()> {
        self.client = Some(client);
        Ok(())
    }
    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("Antares"),
            description: String::from("二维码生成器"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_command(
        &mut self,
        _command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data_str = args.join(" ");
        if data_str == "" {
            return Err(anyhow!("请输入要生成二维码的数据!").into());
        }
        let cfg = self.config.as_ref().unwrap();
        if data_str.len() > cfg.max_string_length as usize {
            return Err(anyhow!("数据的字节数不得超过 {}", cfg.max_string_length).into());
        }
        let encoded =
            qrcode_generator::to_png_to_vec(data_str, qrcode_generator::QrCodeEcc::Low, 1024)?;
        let b64enc = base64::encode(encoded);
        self.client
            .clone()
            .unwrap()
            .quick_send_by_sender(
                &sender,
                format!("[CQ:image,file=base64://{}]", b64enc).as_str(),
            )
            .await?;
        Ok(())
    }
}

export_static_plugin!(PLUGIN_NAME, QRCodePlugin::default());
