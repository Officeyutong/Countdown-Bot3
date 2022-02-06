use config::Config;
use serde::{Deserialize, Serialize};
use url::Url;

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
#[derive(Clone, Debug)]
pub struct SubUrlWrapper {
    url_prefix: Url,
}
impl SubUrlWrapper {
    pub fn new(url: &str) -> Self {
        Self {
            url_prefix: url::Url::parse(url).unwrap(),
        }
    }
    pub fn get_sub_url(&self, sub: &str) -> String {
        let t = if sub.starts_with("/") {
            sub.trim_start_matches("/").to_string()
        } else {
            sub.to_string()
        };
        let suburl = self.url_prefix.join(&t).unwrap();
        return suburl.to_string();
    }
}
