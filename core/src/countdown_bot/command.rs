use super::event::{
    message::{GroupMessageEvent, PrivateMessageEvent},
    EventContainer,
};
use anyhow::anyhow;
use std::{collections::HashMap, sync::Arc};
#[derive(Debug)]
pub struct Command {
    pub command_name: String,
    pub alias: Vec<String>,
    pub description: String,
    pub plugin_name: Option<String>,
    pub group_enabled: bool,
    pub private_enabled: bool,
    pub console_enabled: bool,
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
        }
    }
    // pub fn set_async(self, v: bool) -> Self {
    //     let mut t = Command::from(self);
    //     t.async_command = v;
    //     return t;
    // }
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
    pub command_map: HashMap<String, Arc<Command>>,
    alias_map: HashMap<String, String>,
    curr_plugin_name: String,
}
impl CommandManager {
    pub fn update_plugin_name(&mut self, s: String) {
        self.curr_plugin_name = s;
    }
    pub fn new() -> Self {
        CommandManager {
            alias_map: HashMap::new(),
            command_map: HashMap::new(),
            curr_plugin_name: String::from(""),
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
        self.command_map
            .insert(updated_cmd.command_name.clone(), Arc::new(updated_cmd));
        return Ok(());
    }
}
#[derive(Clone, Debug)]
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
#[derive(Debug, Clone)]
pub enum SenderType {
    Console(ConsoleSender),
    Private(PrivateMessageEvent),
    Group(GroupMessageEvent),
}
