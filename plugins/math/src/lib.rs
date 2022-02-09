use async_trait::async_trait;
use config::MathPluginConfig;
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
use log::debug;

static PLUGIN_NAME: &str = "math";

mod command_impl;
mod config;
mod exec_impl;

struct MathPlugin {
    client: Option<CountdownBotClient>,
    config: Option<MathPluginConfig>,
}

impl Default for MathPlugin {
    fn default() -> Self {
        Self {
            client: None,
            config: None,
        }
    }
}

#[async_trait]
impl BotPlugin for MathPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        debug!("Config: {:#?}", self.config);
        bot.register_command(
            Command::new("solve")
                .group(true)
                .description("解方程组 | solve <未知数1[,未知数2[,...]]> <方程1[,方程2[,...]]>"),
        )?;
        bot.register_command(
            Command::new("integrate")
                .group(true)
                .description("不定积分 | integrate <函数>"),
        )?;
        bot.register_command(
            Command::new("diff")
                .group(true)
                .description("求导 | diff <函数>"),
        )?;
        bot.register_command(
            Command::new("latex")
                .group(true)
                .description("渲染LaTeX | latex <文本>"),
        )?;
        bot.register_command(
            Command::new("series")
                .group(true)
                .description("级数展开 | series <展开点> <函数>"),
        )?;
        bot.register_command(
            Command::new("plot")
                .group(true)
                .description("绘制函数图像 | plot <起始点> <终点> <函数1[,函数2[,...]]>"),
        )?;
        bot.register_command(
            Command::new("plotpe")
                .group(true)
                .description("绘制参数方程函数图像 | plotpe <参数起始点(参数符号为t)> <参数重点> <x方程1:y方程1[,x方程2:y方程2[,...]]>"),
        )?;
        bot.register_command(
            Command::new("factor")
                .group(true)
                .description("分解因式 | factor <式子>"),
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
            author: String::from("officeyutong"),
            description: String::from("sympy相关功能封装"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    async fn on_command(
        &mut self,
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        return self.dispatch_command(command, args, sender).await;
    }
}

impl MathPlugin {
    pub async fn dispatch_command(
        &mut self,
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut image_only = false;
        let execute_result = match command.as_str() {
            "solve" => self.command_solve(args).await?,
            "factor" => self.command_factor(args).await?,
            "integrate" => self.command_integrate(args).await?,
            "diff" => self.command_diff(args).await?,
            "series" => self.command_series(args).await?,
            "plot" => {
                image_only = true;
                self.command_plot(args).await?
            }
            "plotpe" => {
                image_only = true;
                self.command_plotpe(args).await?
            }
            "latex" => {
                image_only = true;
                self.command_latex(args).await?
            }
            _ => todo!(),
        };
        debug!("Execute result:\n{:#?}", execute_result);
        if image_only {
            self.client
                .as_ref()
                .unwrap()
                .quick_send_by_sender(
                    sender,
                    &format!("[CQ:image,file=base64://{}]", execute_result.image),
                )
                .await?;
        } else {
            execute_result
                .send_to(sender, self.client.as_ref().unwrap())
                .await?;
        }
        if execute_result.error.trim() != "" {
            self.client
                .as_ref()
                .unwrap()
                .quick_send_by_sender(sender, &format!("程序标准错误:\n{}", execute_result.error))
                .await?;
        }
        return Ok(());
    }
}

export_static_plugin!(PLUGIN_NAME, MathPlugin::default());
