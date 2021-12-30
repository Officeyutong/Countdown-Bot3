use anyhow::anyhow;
use async_trait::async_trait;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::{CountdownBotClient, ResultType},
        command::{Command, SenderType},
        event::EventContainer,
        plugin::{BotPlugin, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    export_static_plugin,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use trust_dns_client::{
    client::{AsyncClient, ClientHandle},
    rr::{DNSClass, Name, RData, Record, RecordType},
};

static PLUGIN_NAME: &str = "dns_query";

#[derive(Deserialize, Serialize)]
pub struct DNSQueryConfig {
    pub dns_server: String,
}

impl Default for DNSQueryConfig {
    fn default() -> Self {
        Self {
            dns_server: "114.114.114.114:53".to_string(),
        }
    }
}

#[derive(Default)]
struct DNSQueryPlugin {
    client: Option<CountdownBotClient>,
    config: Option<DNSQueryConfig>,
}

#[async_trait]
impl BotPlugin for DNSQueryPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        bot.register_command(
            Command::new("dns")
                .enable_all()
                .description("查询域名解析记录 | dns <主机> [A|MX|NS|CNAME]"),
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
    async fn on_disable(&mut self) -> HookResult<()> {
        Ok(())
    }
    fn get_meta(&self) -> PluginMeta {
        PluginMeta {
            author: String::from("Antares"),
            description: String::from("DNS查询"),
            version: String::from("2.0"),
        }
    }
    async fn on_event(&mut self, _event: EventContainer) -> HookResult<()> {
        Ok(())
    }

    async fn on_state_hook(&mut self) -> HookResult<String> {
        Ok(String::new())
    }
    async fn on_schedule_loop(&mut self, _name: &str) -> HookResult<()> {
        Ok(())
    }

    async fn on_command(
        &mut self,
        _command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = self.config.as_ref().unwrap();
        let query_message = handle_dns_query(cfg.dns_server.as_str(), &args).await?;
        self.client
            .clone()
            .unwrap()
            .quick_send_by_sender(&sender, query_message.as_str())
            .await?;
        Ok(())
    }
}

export_static_plugin!(PLUGIN_NAME, DNSQueryPlugin::default());

async fn handle_dns_query(dns_server: &str, args: &Vec<String>) -> ResultType<String> {
    use tokio::net::UdpSocket;
    use trust_dns_client::udp::UdpClientStream;
    if args.len() == 0 {
        return Err(anyhow!("请输入要查询的域名!").into());
    }
    let stream = UdpClientStream::<UdpSocket>::new(dns_server.parse()?);
    let (mut client, background) = AsyncClient::connect(stream).await?;
    let handle = tokio::spawn(background);
    let mut buf = String::new();
    if let Some(query_mode) = args.get(1) {
        handle_general_query(&mut client, args[0].as_str(), &mut buf, query_mode).await?;
    } else {
        for mode in &["A", "MX", "NS", "CNAME"] {
            handle_general_query(&mut client, args[0].as_str(), &mut buf, mode).await?;
        }
    }
    handle.abort();
    return Ok(buf);
}

async fn handle_general_query(
    client: &mut AsyncClient,
    domain: &str,
    out: &mut String,
    query_type_string: &str,
) -> ResultType<()> {
    let query_type = match query_type_string {
        "A" => RecordType::A,
        "MX" => RecordType::MX,
        "NS" => RecordType::NS,
        "CNAME" => RecordType::CNAME,
        _ => {
            return Err(anyhow!("未知查询模式: {}", query_type_string).into());
        }
    };
    let resp = client
        .query(Name::from_str(domain)?, DNSClass::IN, query_type)
        .await?;
    out.push_str(format!("{} 记录查询结果:\n", query_type_string).as_str());
    let answers = resp.answers();
    if answers.len() == 0 {
        out.push_str(format!("未查询到 {} 记录!\n", query_type_string).as_str());
    } else {
        match &query_type {
            RecordType::A => handle_A_query(answers, out).await?,
            RecordType::MX => handle_MX_query(answers, out).await?,
            RecordType::NS => handle_NS_query(answers, out).await?,
            RecordType::CNAME => handle_CNAME_query(answers, out).await?,
            _ => todo!(),
        }
    }
    out.push_str("\n");
    return Ok(());
}
#[allow(non_snake_case)]
async fn handle_A_query(records: &[Record], out: &mut String) -> ResultType<()> {
    for record in records.iter() {
        match record.rdata() {
            RData::A(addr) => {
                out.push_str(addr.to_string().as_str());
                out.push_str("\n");
            }
            _ => {}
        };
    }
    return Ok(());
}
#[allow(non_snake_case)]
async fn handle_MX_query(records: &[Record], out: &mut String) -> ResultType<()> {
    for record in records.iter() {
        match record.rdata() {
            RData::MX(mx_data) => {
                out.push_str(
                    format!(
                        "MX preference = {}, main exchanger = {}\n",
                        mx_data.preference(),
                        mx_data.exchange()
                    )
                    .as_str(),
                );
            }
            _ => {}
        };
    }
    return Ok(());
}
#[allow(non_snake_case)]
async fn handle_NS_query(records: &[Record], out: &mut String) -> ResultType<()> {
    for record in records.iter() {
        match record.rdata() {
            RData::NS(name) => {
                out.push_str(format!("NS = {}\n", name).as_str());
            }
            _ => {}
        };
    }
    return Ok(());
}
#[allow(non_snake_case)]
async fn handle_CNAME_query(records: &[Record], out: &mut String) -> ResultType<()> {
    for record in records.iter() {
        match record.rdata() {
            RData::CNAME(name) => {
                out.push_str(format!("CNAME = {}\n", name).as_str());
            }
            _ => {}
        };
    }
    return Ok(());
}
