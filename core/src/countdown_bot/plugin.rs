use super::bot;
use super::client::CountdownBotClient;
use super::command::SenderType;
use super::event::EventContainer;
use libloading::Library;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::io;
use std::rc::Rc;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
pub type BotPluginWrapped = Arc<Mutex<dyn BotPlugin + Send>>;
// pub type BotPluginWrapped = Arc<dyn BotPlugin + Send>;
pub type HookResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Debug)]
pub struct PluginMeta {
    pub author: String,
    pub description: String,
    pub version: String,
}
#[async_trait::async_trait()]
pub trait BotPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
        handle: tokio::runtime::Handle,
    ) -> HookResult<()>;
    fn on_before_start(
        &mut self,
        bot: &mut bot::CountdownBot,
        client: CountdownBotClient,
    ) -> HookResult<()>;
    async fn on_disable(&mut self) -> HookResult<()>;
    fn get_meta(&self) -> PluginMeta;
    async fn on_event(&mut self, event: EventContainer) -> bool;
    async fn on_command(
        &mut self,
        command: String,
        args: Vec<String>,
        sender: &SenderType,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn on_state_hook(&mut self) -> String;
    async fn on_schedule_loop(&mut self, name: &str);
}
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
pub type PluginWrapperArc = Arc<PluginWrapper>;
pub struct PluginWrapper {
    pub meta: PluginMeta,
    pub plugin_instance: BotPluginWrapped,
    // pub library: Rc<Library>,
    pub load_source: PluginLoadSource,
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
    ) -> Result<(), Box<dyn Error>> {
        let mut registrar = LocalPluginRegistrar::new(None);
        register_plugin(&mut registrar);
        let plugin_inst = registrar.plugin.unwrap().clone();
        self.plugins.insert(
            registrar.name.clone(),
            Arc::new(PluginWrapper {
                load_source: PluginLoadSource::Static,
                meta: plugin_inst.clone().lock().await.get_meta(),
                plugin_instance: plugin_inst,
            }),
        );
        Ok(())
    }
    pub async unsafe fn load_plugin<P: AsRef<OsStr>>(
        &mut self,
        library_path: P,
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
        let plugin_inst = registrar.plugin.unwrap().clone();
        self.plugins.insert(
            registrar.name.clone(),
            Arc::new(PluginWrapper {
                load_source: PluginLoadSource::Dynamic(registrar.lib.unwrap()),
                meta: plugin_inst.clone().lock().await.get_meta(),
                plugin_instance: plugin_inst,
            }),
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
                std::sync::Arc::new(tokio::sync::Mutex::new($plugin_instance)),
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
