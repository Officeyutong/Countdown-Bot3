use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Local, Timelike};
use log::{error, info, trace};
use tokio::sync::Mutex;

use self::handler::ScheduleLoopHandler;

use super::{bot::StopSignalReceiverType, plugin::BotPluginWrapped};
pub mod handler;
// #[derive(Clone)]
pub struct ScheduleItemWrapper {
    pub time: (u32, u32),
    pub plugin: BotPluginWrapped,
    pub name: String,
    pub last_executed: Option<DateTime<Local>>,
    pub handler: Arc<Mutex<dyn ScheduleLoopHandler>>,
}
// #[derive(Clone)]
pub struct ScheduleLoopManager {
    pub schedules: Vec<ScheduleItemWrapper>,
    current_plugin: Option<BotPluginWrapped>,
    pub stop_signal_receiver: Option<StopSignalReceiverType>,
}
impl ScheduleLoopManager {
    pub fn set_current_plugin(&mut self, plugin_wrapper: BotPluginWrapped) {
        self.current_plugin = Some(plugin_wrapper);
    }
    pub fn set_stop_signal_receiver(&mut self, receiver: StopSignalReceiverType) {
        self.stop_signal_receiver = Some(receiver);
    }
    pub fn new() -> Self {
        Self {
            schedules: vec![],
            current_plugin: None,
            stop_signal_receiver: None,
        }
    }
    pub fn register(
        &mut self,
        time: (u32, u32),
        name: String,
        handler: Arc<Mutex<dyn ScheduleLoopHandler>>,
    ) {
        self.schedules.push(ScheduleItemWrapper {
            name,
            plugin: self.current_plugin.as_ref().unwrap().clone(),
            time,
            last_executed: None,
            handler,
        });
    }
    async fn map_everything(&mut self) {
        trace!("Checking schedule loops..");
        for item in self.schedules.iter_mut() {
            let now = chrono::Local::now();
            let (exp_hour, exp_minute) = item.time;
            if now.time().minute() == exp_minute && now.time().hour() == exp_hour {
                let can_execute = match &item.last_executed {
                    Some(last_executed) => {
                        let diff = now - last_executed.clone();
                        diff.num_hours() >= 23 && diff.num_minutes() >= 50
                    }
                    None => true,
                };
                if can_execute {
                    info!("Ok to execute \"{}\", executing..", item.name);
                    item.last_executed = Some(now.clone());
                    let plugin_inst = item.plugin.clone();
                    let name_cloned = item.name.clone();
                    let handler_ref = item.handler.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handler_ref
                            .lock()
                            .await
                            .on_schedule_loop(name_cloned.as_str(), plugin_inst)
                            .await
                        {
                            error!(
                                "Error handling schedule loop {}:\n{}",
                                name_cloned.as_str(),
                                e
                            );
                        }
                    });
                }
            }
        }
    }
    pub async fn run(mut self) {
        info!("Starting schedule loop...");
        let mut receiver = self.stop_signal_receiver.clone().unwrap();
        loop {
            tokio::select! {
                _ = receiver.changed() => {
                    if *receiver.borrow() {
                        info!("Shutting down schedule loop handler..");
                        break;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    self.map_everything().await;
                }
            }
        }
    }
}
