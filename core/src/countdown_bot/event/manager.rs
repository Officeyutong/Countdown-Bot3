use crate::countdown_bot::client::ResultType;
use crate::countdown_bot::plugin::BotPluginWrapped;

use super::OOPEventContainer;
use async_trait::async_trait;
use log::error;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

pub type WrappedOOPEventContainer = Arc<RwLock<OOPEventContainer>>;
#[async_trait]
pub trait EventListener: downcast_rs::Downcast + Sync + Send {
    async fn on_event(
        &mut self,
        event: WrappedOOPEventContainer,
        plugin: BotPluginWrapped,
    ) -> ResultType<()>;
}
downcast_rs::impl_downcast!(EventListener);
// pub type ListenerWrapper = Arc<Mutex<dyn EventListener>>;

struct EventListenerWrapper {
    pub(crate) plugin: BotPluginWrapped,
    pub(crate) listener: Arc<Mutex<dyn EventListener>>,
}
pub struct EventManager {
    // listeners: Vec<ListenerWrapper>,
    listeners: HashMap<TypeId, Vec<EventListenerWrapper>>,
}
impl EventManager {
    pub fn new() -> EventManager {
        EventManager {
            listeners: HashMap::new(),
        }
    }

    pub fn register_listener(
        &mut self,
        event_type: TypeId,
        listener: Arc<Mutex<dyn EventListener>>,
        plugin: BotPluginWrapped,
    ) {
        // self.listeners.push(listener);
        if !self.listeners.contains_key(&event_type) {
            self.listeners.insert(event_type, vec![]);
        }
        self.listeners
            .get_mut(&event_type)
            .unwrap()
            .push(EventListenerWrapper { plugin, listener });
    }
    pub async fn dispatch_event(&self, event: WrappedOOPEventContainer) {
        let tid = event.read().await.event.type_id();
        if let Some(listeners) = self.listeners.get(&tid) {
            for item in listeners.iter() {
                let plugin = item.plugin.clone();
                let event = event.clone();
                let listener = item.listener.clone();
                tokio::spawn(async move {
                    let raw_value = event.read().await.raw_value.clone();
                    let handle_result = listener.lock().await.on_event(event, plugin).await;
                    if let Err(e) = handle_result {
                        error!(
                            "Error occured when handling event:\n{}\n{}",
                            e,
                            raw_value.to_string()
                        );
                    }
                });
            }
        }
    }
}
