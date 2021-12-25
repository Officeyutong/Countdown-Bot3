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

#[derive(Debug)]
pub struct PluginMeta {
    pub author: String,
    pub description: String,
    pub version: f64,
}
#[async_trait::async_trait]
pub trait BotPlugin {
    fn on_enable(
        &mut self,
        bot: &mut bot::CountdownBot,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
    fn on_disable(
        &mut self,
        bot: &mut bot::CountdownBot,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
    fn get_meta(&self) -> PluginMeta;
    async fn on_event(&mut self, event: EventContainer) -> bool;
    async fn on_command(&mut self, command: String, args: Vec<String>, sender: SenderType, client: CountdownBotClient);
}

pub struct PluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: unsafe extern "C" fn(&mut dyn PluginRegistrar),
    pub name: &'static str,
}

pub trait PluginRegistrar {
    fn register_plugin(&mut self, plugin: BotPluginWrapped);
}

struct LocalPluginRegistrar {
    plugin: Option<BotPluginWrapped>,
    lib: Rc<Library>,
    name: String,
}
impl LocalPluginRegistrar {
    pub fn new(lib: Rc<Library>) -> LocalPluginRegistrar {
        LocalPluginRegistrar {
            lib: lib,
            name: String::from(""),
            plugin: None,
        }
    }
}
impl PluginRegistrar for LocalPluginRegistrar {
    fn register_plugin(&mut self, plugin: BotPluginWrapped) {
        self.plugin = Some(plugin);
    }
}
pub struct PluginWrapper {
    pub meta: PluginMeta,
    pub plugin: BotPluginWrapped,
    pub library: Rc<Library>,
}
unsafe impl Send for PluginWrapper {}
#[derive(Default)]
pub struct PluginManager {
    pub plugins: HashMap<String, Arc<Mutex<PluginWrapper>>>,
    // pub libraries: HashMap<String, Rc<Library>>,
}
impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager::default()
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
        let mut registrar = LocalPluginRegistrar::new(Rc::clone(&library));
        (plugin_decl.register)(&mut registrar);
        registrar.name = String::from(plugin_decl.name);
        let plugin_inst = registrar.plugin.unwrap().clone();
        self.plugins.insert(
            registrar.name.clone(),
            Arc::new(Mutex::new(PluginWrapper {
                library: registrar.lib,
                meta: plugin_inst.clone().lock().await.get_meta(),
                plugin: plugin_inst,
            })),
        );
        // self.libraries.insert(registrar.name, registrar.lib);
        Ok(())
    }
}

#[macro_export]
macro_rules! export_plugin {
    ($register:expr, $name:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static plugin_declaration: $crate::countdown_bot::plugin::PluginDeclaration =
            $crate::countdown_bot::plugin::PluginDeclaration {
                rustc_version: $crate::countdown_bot::bot::RUSTC_VERSION,
                core_version: $crate::countdown_bot::bot::CORE_VERSION,
                register: $register,
                name: $name,
            };
    };
}
