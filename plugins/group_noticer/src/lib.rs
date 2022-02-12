use async_trait::async_trait;
use countdown_bot3::{
    countdown_bot::{
        bot,
        client::{CountdownBotClient, ResultType},
        event::{
            manager::{EventListener, WrappedOOPEventContainer},
            notice::{
                GroupMembersIncreaseEvent, GroupMembersReduceEvent, GroupMembersReduceSubType,
            },
        },
        plugin::{BotPlugin, BotPluginWrapped, HookResult, PluginMeta},
        utils::load_config_or_save_default,
    },
    export_static_plugin,
};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
static PLUGIN_NAME: &str = "group_noticer";

#[derive(Deserialize, Serialize)]
pub struct GroupNoticerConfig {
    pub welcome_message: String,
    pub disable_groups: Vec<i64>,
}

impl Default for GroupNoticerConfig {
    fn default() -> Self {
        Self {
            welcome_message: String::from("{at}\n哇，你来啦，要玩的开心哦！"),
            disable_groups: vec![],
        }
    }
}

#[derive(Default)]
struct GroupNoticerPlugin {
    client: Option<CountdownBotClient>,
    config: Option<GroupNoticerConfig>,
}

#[async_trait]
impl BotPlugin for GroupNoticerPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        self.config = Some(load_config_or_save_default::<GroupNoticerConfig>(
            &bot.ensure_plugin_data_dir(PLUGIN_NAME)?,
        )?);
        bot.register_event_handler(TypeId::of::<GroupMembersIncreaseEvent>(), MyEventHandler {});
        bot.register_event_handler(TypeId::of::<GroupMembersReduceEvent>(), MyEventHandler {});

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
            description: String::from("入群退群通知器"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

struct MyEventHandler;
#[async_trait::async_trait]
impl EventListener for MyEventHandler {
    async fn on_event(
        &mut self,
        event: WrappedOOPEventContainer,
        plugin: BotPluginWrapped,
    ) -> ResultType<()> {
        let plugin_guard = plugin.read().await;
        let casted = plugin_guard.downcast_ref::<GroupNoticerPlugin>().unwrap();
        let config = casted.config.as_ref().unwrap();
        let client = casted.client.as_ref().unwrap();
        let event_obj = event.read().await.event.clone();
        if let Some(inc) = event_obj.downcast_ref::<GroupMembersIncreaseEvent>() {
            let group_id = inc.group_id;
            if !config.disable_groups.contains(&group_id) {
                client
                    .send_group_msg(
                        group_id,
                        config
                            .welcome_message
                            .replace("{at}", format!("[CQ:at,qq={}]", inc.user_id).as_str())
                            .as_str(),
                        false,
                    )
                    .await?;
            }
        };
        if let Some(dec) = event_obj.downcast_ref::<GroupMembersReduceEvent>() {
            let group_id = dec.group_id;
            let uid = dec.user_id;
            if !config.disable_groups.contains(&group_id) {
                let stranger_info = client.get_stranger_info(uid, false).await?;
                let str = match dec.sub_type {
                    GroupMembersReduceSubType::Leave => {
                        format!(
                            "用户 {}({}) 已退出本群",
                            stranger_info.user_id, stranger_info.nickname
                        )
                    }
                    GroupMembersReduceSubType::Kick => {
                        format!(
                            "用户 {}({}) 已被踢出本群",
                            stranger_info.user_id, stranger_info.nickname
                        )
                    }
                    GroupMembersReduceSubType::KickMe => return Ok(()),
                };
                client.send_group_msg(group_id, str.as_str(), true).await?;
            }
        };

        return Ok(());
    }
}

export_static_plugin!(PLUGIN_NAME, GroupNoticerPlugin::default());
