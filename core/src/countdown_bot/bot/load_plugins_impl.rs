use super::CountdownBot;
use crate::countdown_bot::plugin::PluginWrapper;
use std::sync::Arc;
use tokio::sync::Mutex;

use log::{debug, error, info};

impl CountdownBot {
    pub async fn load_plugins(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use std::env::consts::{DLL_EXTENSION, DLL_PREFIX};
        let path1 = {
            let exe = std::env::current_exe()?;
            std::path::PathBuf::from(exe.parent().unwrap())
        };
        let path2 = self.sys_root.clone();
        let load_path: Vec<std::path::PathBuf> = vec![path1, path2];
        let mut libs: Vec<std::path::PathBuf> = vec![];
        for path in load_path.iter() {
            info!("Listing libraries under: {}", path.display());
            for file in std::fs::read_dir(path).unwrap() {
                if let Ok(f) = file {
                    let lib_path = f.path();
                    if let Some(file_name) = lib_path.file_name() {
                        if let Some(s) = file_name.to_str() {
                            if let Ok(tp) = f.file_type() {
                                if !tp.is_file() {
                                    debug!("Ignoring \"{}\", it's not a file.", s);
                                    continue;
                                }
                            }
                            if s.ends_with(DLL_EXTENSION) && s.starts_with(DLL_PREFIX) {
                                libs.push(f.path());
                                debug!("Catched \"{}\"", s);
                            } else {
                                debug!("Ignoring \"{}\", because its pathname doesn't begin with \"{}\" or end with \"{}\"",s,DLL_PREFIX,DLL_EXTENSION);
                            }
                        }
                    }
                }
            }
        }
        info!("Plugins to load:");
        for item in libs.iter() {
            info!("{}", item.display());
        }
        for item in libs.iter() {
            if let Err(e) = unsafe { self.plugin_manager.load_plugin(item.as_os_str()).await } {
                error!("Error loading: {}", item.display());
                error!("{}", e);
            }
        }
        let mut plugins: Vec<(String, Arc<Mutex<PluginWrapper>>)> = vec![];
        for (name, plugin) in self.plugin_manager.plugins.iter() {
            if self.preserved_plugin_names.contains(name) {
                panic!("Preserved plugin name: {}", name);
            }
            plugins.push((String::from(name), plugin.clone()));
        }
        for (name, plugin) in plugins.iter() {
            info!("Loading {}", name);
            self.command_manager.update_plugin_name(name.clone());
            let plugin_locked = plugin.lock().await;
            if let Err(e) = plugin_locked.plugin.lock().await.on_enable(self) {
                error!("Error enablng: {}", name);
                error!("{}", e);
            } else {
                info!("Loaded: name={}, meta={:?}", name, plugin_locked.meta);
            };
        }
        info!(
            "Registered {} commands",
            self.command_manager.command_map.len()
        );
        debug!(
            "Registered commands: {:?}",
            self.command_manager.command_map
        );
        return Ok(());
    }
}
