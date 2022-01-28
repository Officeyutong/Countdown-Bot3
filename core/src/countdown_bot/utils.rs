use config::Config;
use serde::{Deserialize, Serialize};

pub fn load_config_or_save_default<'a, T>(data_path: &std::path::PathBuf) -> anyhow::Result<T>
where
    T: Serialize + Deserialize<'a> + Default,
{
    let config_file = data_path.join("config.yaml");
    let mut cfg = Config::new();
    if !config_file.exists() {
        std::fs::write(
            &config_file,
            serde_yaml::to_string(&T::default())?.as_bytes(),
        )?;
    }
    cfg.merge(config::Config::try_from(&T::default())?)?;
    cfg.merge(config::File::from(config_file))?;
    Ok(cfg.try_into()?)
}
