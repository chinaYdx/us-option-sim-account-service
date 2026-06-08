use std::path::PathBuf;

use x_com_lib::x_core::config;

const DEFAULT_DATA_PATH: &str = "./data";

#[derive(Clone, Debug)]
pub struct SimAccountConfig {
    pub data_path: String,
}

impl SimAccountConfig {
    pub fn load() -> Self {
        let data_path = config::get_str("path")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_DATA_PATH)
            .to_owned();
        SimAccountConfig { data_path }
    }

    pub fn db_path(&self) -> PathBuf {
        PathBuf::from(&self.data_path)
            .join("us-option-sim-account")
            .join("account.db")
    }
}
