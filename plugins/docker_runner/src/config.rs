use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LanguageSetting {
    pub source_filename: String,
    pub executable_filename: String,
    pub compile: String,
    pub run: String,
}
impl LanguageSetting {
    pub fn source_file(&self, name: &str) -> String {
        return self.source_filename.clone().replace("{name}", name);
    }
    pub fn executable_file(&self, name: &str) -> String {
        return self.executable_filename.clone().replace("{name}", name);
    }
    pub fn compile_arg(&self, source: &str, target: &str) -> String {
        return self
            .compile
            .clone()
            .replace("{source}", source)
            .replace("{target}", target);
    }
    pub fn run_arg(&self, target: &str) -> String {
        return self.run.clone().replace("{target}", target);
    }
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DockerRunnerConfig {
    pub docker_image: String,
    pub outout_length_limit: i32,
    pub execute_time_limit: i32,
    pub input_expire_after: i64,
    pub new_line_count_limit: i32,
    pub language_setting: HashMap<String, LanguageSetting>,
    pub blacklist_users: Vec<i64>,
}
impl Default for DockerRunnerConfig {
    fn default() -> Self {
        Self {
            docker_image: "python".to_string(),
            execute_time_limit: 2000,
            input_expire_after: 1000 * 60 * 60,
            new_line_count_limit: 5,
            outout_length_limit: 200,
            language_setting: HashMap::from([
                (
                    "python".to_string(),
                    LanguageSetting {
                        source_filename: "{name}.py_".to_string(),
                        executable_filename: "{name}.py".to_string(),
                        compile: "cp {source} {target}".to_string(),
                        run: "python3 {target}".to_string(),
                    },
                ),
                (
                    "cpp".to_string(),
                    LanguageSetting {
                        source_filename: "{name}.cpp".to_string(),
                        executable_filename: "{name}.out".to_string(),
                        compile: "g++ -fdiagnostics-color=never {source} -o {target}".to_string(),
                        run: "./{target}".to_string(),
                    },
                ),
                (
                    "c".to_string(),
                    LanguageSetting {
                        source_filename: "{name}.c".to_string(),
                        executable_filename: "{name}.out".to_string(),
                        compile: "gcc -fdiagnostics-color=never {source} -o {target}".to_string(),
                        run: "./{target}".to_string(),
                    },
                ),
                (
                    "bash".to_string(),
                    LanguageSetting {
                        source_filename: "{name}.sh_".to_string(),
                        executable_filename: "{name}.sh".to_string(),
                        compile: "cp {source} {target}".to_string(),
                        run: "bash {target}".to_string(),
                    },
                ),
                (
                    "rust".to_string(),
                    LanguageSetting {
                        source_filename: "{name}.rs".to_string(),
                        executable_filename: "{name}.out".to_string(),
                        compile: "rustc {source} -o {target}".to_string(),
                        run: "./{target}".to_string(),
                    },
                ),
                (
                    "haskell".to_string(),
                    LanguageSetting {
                        source_filename: "{name}.hs".to_string(),
                        executable_filename: "{name}.out".to_string(),
                        compile: "ghc {source} -o {target}".to_string(),
                        run: "./{target}".to_string(),
                    },
                ),
            ]),
            blacklist_users: vec![],
        }
    }
}
