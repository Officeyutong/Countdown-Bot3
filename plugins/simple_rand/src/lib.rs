use anyhow::anyhow;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::CountdownBotClient,
        command::{Command, SenderType},
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    initialize_plugin_logger,
    // initialize_plugin_logger,
};
use rand::{
    prelude::{SliceRandom, StdRng},
    SeedableRng,
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};
#[derive(Deserialize, Serialize, Debug)]
struct SimpleRandConfig {
    pub max_number_count: u32,
}
impl Default for SimpleRandConfig {
    fn default() -> Self {
        Self {
            max_number_count: 20,
        }
    }
}
static PLUGIN_NAME: &str = "simple_rand";

struct SimpleRandPlugin {
    client: Option<CountdownBotClient>,
    plugin_data_root: Option<PathBuf>,
    config: Option<SimpleRandConfig>,
}
impl SimpleRandPlugin {
    pub fn new() -> Self {
        Self {
            client: None,
            plugin_data_root: None,
            config: None,
        }
    }
}
#[async_trait::async_trait]
impl BotPlugin for SimpleRandPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        initialize_plugin_logger!(bot);
        self.plugin_data_root = Some(bot.ensure_plugin_data_dir(PLUGIN_NAME)?);
        bot.register_command(
            Command::new("rand")
                .description("生成随机数 | rand <上限> [个数]")
                .console(true)
                .group(true)
                .private(true)
                .guild(true)
                .single_alias("随机"),
        )?;
        bot.register_command(
            Command::new("choice")
                .description("随机选择 | choice <选项1> [选项2 [选项3 [...]]]")
                .console(true)
                .group(true)
                .private(true)
                .guild(true)
                .single_alias("随机选择"),
        )?;
        self.config = Some(load_config_or_save_default::<SimpleRandConfig>(
            &self.plugin_data_root.as_ref().unwrap(),
        )?);
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
            author: String::from("officeyutong"),
            description: String::from("简单随机数实现"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_command(
        &mut self,
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command.as_str() {
            "rand" => self.handle_rand(&args, sender).await?,
            "choice" => self.handle_choice(&args, sender).await?,
            _ => {}
        };
        return Ok(());
    }
}

countdown_bot3::export_static_plugin!(PLUGIN_NAME, SimpleRandPlugin::new());

impl SimpleRandPlugin {
    async fn handle_choice(
        &self,
        args: &Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() < 1 {
            return Err(anyhow!("请输入最少一个选项！").into());
        }
        let max_count = self.config.as_ref().unwrap().max_number_count;
        if args.len() > max_count as usize {
            return Err(anyhow!("最多允许 {} 个随机选项！", max_count).into());
        }
        let mut rng: StdRng = SeedableRng::from_entropy();
        let elem = args.choose(&mut rng).unwrap();
        self.client
            .as_ref()
            .unwrap()
            .quick_send_by_sender(&sender, format!("你的选择结果是：\n{}", elem).as_str())
            .await?;
        return Ok(());
    }
    async fn handle_rand(
        &self,
        args: &Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() == 0 {
            return Err(anyhow!("请输入至少一个参数!").into());
        }
        use num_bigint::{BigInt, RandBigInt, ToBigInt};
        let mut rng: StdRng = SeedableRng::from_entropy();
        let upper = BigInt::from_str(&args[0]).map_err(|_| anyhow!("请输入合法的正整数!"))?;
        if upper <= 0.to_bigint().unwrap() {
            return Err(anyhow!("请输入正整数!").into());
        };

        let count = match args.get(1) {
            Some(s) => u32::from_str_radix(s, 10).map_err(|_| anyhow!("请输入合法的数据个数!"))?,
            None => 1u32,
        };
        if count > self.config.as_ref().unwrap().max_number_count {
            return Err(anyhow!(format!(
                "最多允许生成 {} 个随机数据!",
                self.config.as_ref().unwrap().max_number_count
            ))
            .into());
        }
        let mut output = String::from("生成结果:\n");
        for _ in 0..count {
            output.push_str(
                format!("{}", rng.gen_bigint_range(&1.to_bigint().unwrap(), &upper)).as_str(),
            );
            output.push_str("\n");
        }
        let client = self.client.clone().unwrap();
        client
            .quick_send_by_sender(&sender, output.as_str())
            .await?;
        Ok(())
    }
}
