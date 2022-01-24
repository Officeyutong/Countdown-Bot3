use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MathPluginConfig {
    pub docker_image: String,
    pub default_timeout: i64,
    pub function_count_limit: i32,
    pub matplot_range_length: i32,
    pub latex_packages: Vec<String>,
}

impl Default for MathPluginConfig {
    fn default() -> Self {
        Self {
            docker_image: "python".to_string(),
            default_timeout: 30 * 1000,
            function_count_limit: 4,
            matplot_range_length: 30,
            latex_packages: [
                "amssymb",
                "color",
                "amsthm",
                "multirow",
                "enumerate",
                "amstext",
            ]
            .iter()
            .map(|x| String::from(*x))
            .collect(),
        }
    }
}
