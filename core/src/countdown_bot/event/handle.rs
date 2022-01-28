use super::EventContainer;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::Mutex;

#[async_trait]
pub trait GeneralEventHandler {
    async fn on_event(&mut self, event: &EventContainer);
}
pub type ListenerWrapper = Arc<Mutex<dyn GeneralEventHandler>>;
pub struct EventManager {
    listeners: Vec<ListenerWrapper>,
}
impl EventManager {
    pub fn new() -> EventManager {
        EventManager { listeners: vec![] }
    }
    pub fn register_listener(&mut self, listener: ListenerWrapper) {
        self.listeners.push(listener);
    }
    pub async fn send_event(&self, event: EventContainer) {
        for listener in self.listeners.iter() {
            let mut curr = listener.lock().unwrap();
            curr.on_event(&event).await;
        }
    }
}
