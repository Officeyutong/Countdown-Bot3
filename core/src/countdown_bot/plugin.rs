use super::bot;
use super::client::CountdownBotClient;
use super::command::SenderType;
use super::event::EventContainer;
use downcast_rs::{impl_downcast, DowncastSync};
use libloading::Library;
use log::info;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::io;
use std::rc::Rc;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
pub type BotPluginWrapped = Arc<RwLock<dyn BotPlugin>>;
pub type BotPluginNoSend = Arc<RwLock<dyn BotPlugin>>;
// pub type BotPluginWrapped = Arc<dyn BotPlugin + Send>;
pub type HookResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Debug)]
pub struct PluginMeta {
    pub author: String,
    pub description: String,
    pub version: String,
}
#[async_trait::async_trait()]
pub trait BotPlugin: DowncastSync + Send + Sync {
    fn on_enable(
        &mut self,
        _bot: &mut bot::CountdownBot,
        _handle: tokio::runtime::Handle,
    ) -> HookResult<()> {
        return Ok(());
    }
    fn on_before_start(
        &mut self,
        _bot: &mut bot::CountdownBot,
        _client: CountdownBotClient,
    ) -> HookResult<()> {
        return Ok(());
    }
    async fn on_disable(&mut self) -> HookResult<()> {
        return Ok(());
    }
    fn get_meta(&self) -> PluginMeta;
    async fn on_event(&mut self, _event: EventContainer) -> HookResult<()> {
        return Ok(());
    }
    async fn on_command(
        &mut self,
        _command: String,
        _args: Vec<String>,
        _sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        return Ok(());
    }
    async fn on_state_hook(&mut self) -> HookResult<String> {
        return Ok(String::new());
    }
    // async fn on_schedule_loop(&mut self, _name: &str) -> HookResult<()> {
    //     return Ok(());
    // }
    // fn will_use_command_handler(&self) -> bool {
    //     return false;
    // }
    fn will_use_event_handler(&self) -> bool {
        return false;
    }
    // fn will_use_loop_handler(&self) -> bool {
    //     return false;
    // }
}
impl_downcast!(sync BotPlugin);
pub type CTypePluginRegisterCallback = unsafe extern "C" fn(&mut dyn PluginRegistrar);
pub type PluginRegisterCallback = fn(&mut dyn PluginRegistrar);
pub struct PluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: CTypePluginRegisterCallback,
}

pub trait PluginRegistrar {
    fn register_plugin(&mut self, name: &str, plugin: BotPluginWrapped);
}

struct LocalPluginRegistrar {
    plugin: Option<BotPluginWrapped>,
    lib: Option<Rc<Library>>,
    name: String,
}
impl LocalPluginRegistrar {
    pub fn new(lib: Option<Rc<Library>>) -> LocalPluginRegistrar {
        LocalPluginRegistrar {
            lib: lib,
            name: String::from(""),
            plugin: None,
        }
    }
}
impl PluginRegistrar for LocalPluginRegistrar {
    fn register_plugin(&mut self, name: &str, plugin: BotPluginWrapped) {
        self.plugin = Some(plugin);
        self.name = name.to_string();
    }
}
pub type PluginWrapperArc = Arc<RwLock<PluginWrapper>>;
pub struct PluginWrapper {
    pub(crate) meta: PluginMeta,
    pub(crate) plugin_instance: BotPluginWrapped,
    // pub library: Rc<Library>,
    pub(crate) load_source: PluginLoadSource,
    // pub(crate) use_command_handler: bool,
    #[allow(dead_code)]
    pub(crate) use_event_handler: bool,
    // #[allow(dead_code)]
    // pub(crate) use_loop_handler: bool,
}
pub enum PluginLoadSource {
    Static,
    Dynamic(Rc<Library>),
}
unsafe impl Send for PluginWrapper {}
#[derive(Default)]
pub struct PluginManager {
    pub plugins: HashMap<String, PluginWrapperArc>,
    // pub libraries: HashMap<String, Rc<Library>>,
}
impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager::default()
    }
    pub async fn load_static_plugin(
        &mut self,
        register_plugin: PluginRegisterCallback,
        ignored_plugins: &Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        let mut registrar = LocalPluginRegistrar::new(None);
        register_plugin(&mut registrar);
        if ignored_plugins.contains(&registrar.name) {
            info!("Ignoring: {}", registrar.name);
            return Ok(());
        }
        let plugin_inst = registrar.plugin.unwrap().clone();
        let plugin_obj_guard = plugin_inst.as_ref().read().await;
        self.plugins.insert(
            registrar.name.clone(),
            Arc::new(RwLock::new(PluginWrapper {
                load_source: PluginLoadSource::Static,
                meta: plugin_obj_guard.get_meta(),
                plugin_instance: plugin_inst.clone(),
                // use_command_handler: plugin_obj_guard.will_use_command_handler(),
                use_event_handler: plugin_obj_guard.will_use_event_handler(),
                // use_loop_handler: plugin_obj_guard.will_use_loop_handler(),
            })),
        );
        Ok(())
    }
    pub async unsafe fn load_plugin<P: AsRef<OsStr>>(
        &mut self,
        library_path: P,
        ignored_plugins: &Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        let library = Rc::new(Library::new(library_path)?);
        let plugin_decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")?
            .read();
        if plugin_decl.rustc_version != bot::RUSTC_VERSION {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Unexpected rustc_version, expected {}, found {}",
                    bot::RUSTC_VERSION,
                    plugin_decl.rustc_version
                ),
            )));
        }
        if plugin_decl.core_version != bot::CORE_VERSION {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Unexpected core_version, expected {}, found {}",
                    bot::CORE_VERSION,
                    plugin_decl.core_version
                ),
            )));
        }
        let mut registrar = LocalPluginRegistrar::new(Some(Rc::clone(&library)));
        (plugin_decl.register)(&mut registrar);
        // registrar.name = String::from(plugin_decl.name);
        if ignored_plugins.contains(&registrar.name) {
            info!("Ignoring plugin: {}", registrar.name);
            return Ok(());
        }
        let plugin_inst = registrar.plugin.unwrap().clone();
        let plugin_obj_guard = plugin_inst.as_ref().read().await;
        self.plugins.insert(
            registrar.name.clone(),
            Arc::new(RwLock::new(PluginWrapper {
                load_source: PluginLoadSource::Dynamic(registrar.lib.unwrap()),
                meta: plugin_obj_guard.get_meta(),
                plugin_instance: plugin_inst.clone(),
                // use_command_handler: plugin_obj_guard.will_use_command_handler(),
                use_event_handler: plugin_obj_guard.will_use_event_handler(),
                // use_loop_handler: plugin_obj_guard.will_use_loop_handler(),
            })),
        );
        // self.libraries.insert(registrar.name, registrar.lib);
        Ok(())
    }
}

#[macro_export]
macro_rules! export_static_plugin {
    ($name:expr, $plugin_instance:expr) => {
        pub fn plugin_register(
            registrar: &mut dyn countdown_bot3::countdown_bot::plugin::PluginRegistrar,
        ) {
            registrar.register_plugin(
                $name,
                std::sync::Arc::new(tokio::sync::RwLock::new($plugin_instance)),
            );
        }
    };
}
#[macro_export]
macro_rules! export_plugin {
    ($name:expr, $plugin_instance:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static plugin_declaration: $crate::countdown_bot::plugin::PluginDeclaration =
            $crate::countdown_bot::plugin::PluginDeclaration {
                rustc_version: $crate::countdown_bot::bot::RUSTC_VERSION,
                core_version: $crate::countdown_bot::bot::CORE_VERSION,
                register: __c_plugin_register,
            };
        #[allow(improper_ctypes_definitions)]
        extern "C" fn __c_plugin_register(
            registrar: &mut dyn countdown_bot3::countdown_bot::plugin::PluginRegistrar,
        ) {
            registrar.register_plugin(
                $name,
                std::sync::Arc::new(tokio::sync::Mutex::new($plugin_instance)),
            );
        }
    };
}

#[macro_export]
macro_rules! initialize_plugin_logger {
    ($bot:expr) => {
        log::set_logger($bot.get_logger()).ok();
        log::set_max_level($bot.get_max_log_level());
    };
}
