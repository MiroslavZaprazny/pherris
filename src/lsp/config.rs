use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Runtime {
    PhpCli,
    Docker,
    DockerCompose,
    Ts,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeOptions {
    pub runtime: Option<Runtime>,
    pub docker_image: Option<String>,
    pub php_bin_path: Option<String>,
}

impl InitializeOptions {
    pub fn default() -> Self {
        Self {
            runtime: None,
            docker_image: None,
            php_bin_path: None,
        }
    }
}
