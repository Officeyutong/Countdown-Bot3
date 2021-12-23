use super::bot;
use libloading::Library;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::io;
use std::rc::Rc;
use std::result::Result;
pub struct PluginMeta {
    pub author: String,
    pub description: String,
    pub version: f64,
    pub name: String,
}
pub trait BotPlugin {
    fn on_enable(
        &self,
        bot: &mut bot::CountdownBot,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
    fn on_disable(
        &self,
        bot: &mut bot::CountdownBot,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
    fn get_meta(&self) -> PluginMeta;
}

pub struct PluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: unsafe extern "C" fn(&mut dyn PluginRegistrar),
}

pub trait PluginRegistrar {
    fn register_plugin(&mut self, name: &str, plugin: Box<dyn BotPlugin>);
}

pub struct BotPluginProxy {
    plugin: Box<dyn BotPlugin>,
    _lib: Rc<Library>,
}
impl BotPlugin for BotPluginProxy {
    fn on_enable(&self, bot: &mut bot::CountdownBot) -> Result<(), Box<(dyn std::error::Error)>> {
        self.plugin.on_enable(bot)?;
        return Ok(());
    }
    fn on_disable(&self, bot: &mut bot::CountdownBot) -> Result<(), Box<(dyn std::error::Error)>> {
        self.plugin.on_disable(bot)?;
        return Ok(());
    }
    fn get_meta(&self) -> PluginMeta {
        self.plugin.get_meta()
    }
}

struct LocalPluginRegistrar {
    plugin: Option<BotPluginProxy>,
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
    fn register_plugin(
        &mut self,
        name: &str,
        plugin: std::boxed::Box<(dyn BotPlugin + 'static)>,
    ) {
        let proxy = BotPluginProxy {
            plugin: plugin,
            _lib: Rc::clone(&self.lib),
        };
        self.name = String::from(name);
        self.plugin = Some(proxy);
    }
}
#[derive(Default)]
pub struct PluginManager {
    pub plugins: HashMap<String, Rc<BotPluginProxy>>,
    pub libraries: Vec<Rc<Library>>,
}
impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager::default()
    }
    pub unsafe fn load_plugin<P: AsRef<OsStr>>(
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
        self.plugins
            .insert(registrar.name, Rc::new(registrar.plugin.unwrap()));
        self.libraries.push(registrar.lib);
        Ok(())
    }
}

#[macro_export]
macro_rules! export_plugin {
    ($register:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static plugin_declaration: $crate::countdown_bot::plugin::PluginDeclaration =
            $crate::countdown_bot::plugin::PluginDeclaration {
                rustc_version: $crate::countdown_bot::bot::RUSTC_VERSION,
                core_version: $crate::countdown_bot::bot::CORE_VERSION,
                register: $register,
            };
    };
}
