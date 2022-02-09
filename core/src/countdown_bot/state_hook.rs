use std::collections::BTreeSet;

use super::{client::ResultType, plugin::PluginManager};
#[derive(Default)]
pub struct StateHookManager {
    pub hooks: BTreeSet<String>,
    curr_plugin: String,
}

impl StateHookManager {
    pub fn set_curr_plugin(&mut self, plugin: String) {
        self.curr_plugin = plugin;
    }
    pub fn register_state_hook(&mut self) {
        self.hooks.insert(self.curr_plugin.clone());
    }
    pub async fn create_state(&self, plugin_manager: &PluginManager) -> ResultType<String> {
        let mut buf: Vec<String> = vec![];
        for plugin_name in self.hooks.iter() {
            let plugin = plugin_manager.plugins.get(plugin_name).unwrap();
            buf.push(
                plugin
                    .read()
                    .await
                    .plugin_instance
                    .write()
                    .await
                    .on_state_hook()
                    .await?,
            );
        }
        return Ok(buf.join("\n"));
    }
}
