use super::{
    client::ResultType,
    event::{
        message::{GroupMessageEvent, PrivateMessageEvent},
        EventContainer,
    },
};
use anyhow::anyhow;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tokio::sync::Mutex;
#[async_trait::async_trait]
pub trait CommandHandler {
    async fn on_command(
        &mut self,
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
pub type WrappedCommandHandler = Mutex<Box<dyn CommandHandler + Send>>;
// #[derive(Debug)]
pub struct Command {
    pub command_name: String,
    pub alias: Vec<String>,
    pub description: String,
    pub plugin_name: Option<String>,
    pub group_enabled: bool,
    pub private_enabled: bool,
    pub console_enabled: bool,
    pub command_handler: Option<WrappedCommandHandler>,
}

impl Command {
    pub fn new(command_name: &str) -> Self {
        Command {
            command_name: String::from(command_name),
            alias: vec![],
            description: String::from(""),
            plugin_name: None,
            group_enabled: false,
            private_enabled: false,
            console_enabled: false,
            command_handler: None,
        }
    }
    // pub fn set_async(self, v: bool) -> Self {
    //     let mut t = Command::from(self);
    //     t.async_command = v;
    //     return t;
    // }
    pub fn enable_all(self) -> Self {
        Self {
            console_enabled: true,
            private_enabled: true,
            group_enabled: true,
            ..self
        }
    }
    pub fn handler(self, s: Box<dyn CommandHandler + Send>) -> Self {
        let mut t = Command::from(self);
        t.command_handler = Some(Mutex::new(s));
        return t;
    }
    pub fn single_alias(self, s: &str) -> Self {
        let mut t = Command::from(self);
        t.alias.push(String::from(s));
        return t;
    }
    pub fn alias(self, alias: Vec<String>) -> Self {
        let mut t = Command::from(self);
        t.alias = alias;
        return t;
    }
    pub fn description(self, desc: &str) -> Self {
        let mut t = Command::from(self);
        t.description = String::from(desc);
        return t;
    }
    pub fn group(self, v: bool) -> Self {
        let mut t = Command::from(self);
        t.group_enabled = v;
        return t;
    }
    pub fn private(self, v: bool) -> Self {
        let mut t = Command::from(self);
        t.private_enabled = v;
        return t;
    }
    pub fn console(self, v: bool) -> Self {
        let mut t = Command::from(self);
        t.console_enabled = v;
        return t;
    }
    pub fn with_plugin_name(self, v: &String) -> Self {
        let mut t = Command::from(self);
        t.plugin_name = Some(v.clone());
        return t;
    }
}

pub struct CommandManager {
    pub command_map: BTreeMap<String, Arc<Command>>,
    alias_map: HashMap<String, String>,
    curr_plugin_name: String,
    pub last_execute: HashMap<String, HashMap<String, u64>>,
}
impl CommandManager {
    pub fn update_plugin_name(&mut self, s: String) {
        self.curr_plugin_name = s;
    }
    pub fn new() -> Self {
        CommandManager {
            alias_map: HashMap::new(),
            command_map: BTreeMap::new(),
            curr_plugin_name: String::from(""),
            last_execute: HashMap::new(),
        }
    }
    pub fn get_command(
        &self,
        name: &String,
    ) -> std::result::Result<Arc<Command>, Box<dyn std::error::Error>> {
        if let Some(cmd) = self.command_map.get(name) {
            return Ok(cmd.clone());
        } else if let Some(name) = self.alias_map.get(name) {
            return self.get_command(name);
        } else {
            return Err(Box::from(anyhow::anyhow!("Command not found: {}", name)));
        }
    }
    pub fn touch_command_and_test_timeout(
        &mut self,
        name: &str,
        cooldown: u64,
        sender: &SenderType,
    ) -> ResultType<bool> {
        let curr_command = self
            .last_execute
            .get_mut(name)
            .ok_or(anyhow!("Invalid command: {}", name))?;
        let ident = sender.generate_identifier();
        let last_execute = *curr_command.get(&ident).unwrap_or(&0);
        let now_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let diff = now_timestamp - last_execute;
        if diff < cooldown {
            return Ok(false);
        }
        curr_command.insert(ident, now_timestamp);
        return Ok(true);
    }
    pub fn register_command(&mut self, cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
        let updated_cmd = if let None = &cmd.plugin_name {
            Command::from(cmd).with_plugin_name(&self.curr_plugin_name)
        } else {
            cmd
        };
        if updated_cmd.plugin_name.as_ref().unwrap().clone() != self.curr_plugin_name {
            return Err(Box::from(anyhow::anyhow!(
                "Invalid plugin name: {}",
                updated_cmd.plugin_name.unwrap()
            )));
        }
        if self.command_map.contains_key(&updated_cmd.command_name)
            || self.alias_map.contains_key(&updated_cmd.command_name)
        {
            return Err(Box::from(anyhow::anyhow!(
                "Duplicate command name: {}",
                &updated_cmd.command_name
            )));
        }
        for alias in updated_cmd.alias.iter() {
            if self.command_map.contains_key(alias) || self.alias_map.contains_key(alias) {
                return Err(Box::from(anyhow::anyhow!(
                    "Alias \"{}\" duplicates with other command name.",
                    alias
                )));
            }
        }
        for alias in updated_cmd.alias.iter() {
            self.alias_map
                .insert(alias.clone(), updated_cmd.command_name.clone());
        }
        let cmd_name = updated_cmd.command_name.clone();
        self.command_map
            .insert(cmd_name.clone(), Arc::new(updated_cmd));
        self.last_execute
            .insert(cmd_name.clone(), HashMap::new());
        return Ok(());
    }
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ConsoleSender {
    pub line: String,
}
pub enum CommandSender {
    Console(ConsoleSender),
    User(EventContainer),
}
impl CommandSender {
    pub fn parse_sender(&self) -> Result<SenderType, Box<dyn std::error::Error>> {
        match &self {
            CommandSender::Console(sen) => Ok(SenderType::Console(sen.clone())),
            CommandSender::User(evt) => match &evt.event {
                super::event::Event::Message(msg_evt) => match msg_evt {
                    super::event::message::MessageEvent::Private(private_evt) => {
                        Ok(SenderType::Private(private_evt.clone()))
                    }
                    super::event::message::MessageEvent::Group(group_evt) => {
                        Ok(SenderType::Group(group_evt.clone()))
                    }
                    super::event::message::MessageEvent::Unknown => Err(Box::from(anyhow!("?"))),
                },
                _ => Err(Box::from(anyhow!("MessageEvent expected"))),
            },
        }
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SenderType {
    Console(ConsoleSender),
    Private(PrivateMessageEvent),
    Group(GroupMessageEvent),
}

impl SenderType {
    pub fn generate_identifier(&self) -> String {
        match self {
            SenderType::Console(_) => "console".to_string(),
            SenderType::Private(v) => format!("private:{}", v.user_id),
            SenderType::Group(v) => format!("group:{}", v.user_id),
        }
    }
}
