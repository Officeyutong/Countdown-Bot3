use crate::countdown_bot::plugin::{BotPluginWrapped, HookResult};

#[async_trait::async_trait]
pub trait ScheduleLoopHandler: Send + Sync {
    async fn on_schedule_loop(&mut self, name: &str, plugin: BotPluginWrapped) -> HookResult<()>;
}
