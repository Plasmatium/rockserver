use axum::Json;
use axum::extract::Query;
use axum::http::uri::InvalidUri;
use hyper::Uri;
use serde::{Deserialize, Serialize};
use tracing::info;
use std::fs;
use anyhow::Result;
use anyhow::anyhow;

pub static mut G_CONFIG: Config = Config{proxy: None, status_code_threshold: None, enabled: None};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub proxy: Option<String>,
    pub status_code_threshold: Option<u16>,
    pub enabled: Option<bool>,
}

impl Config {
    pub fn from_file(fname: &str) -> Self {
        let config_content =
            fs::read_to_string(fname).expect(&format!("failed to read from file: {fname}"));

        let ret: Config = serde_yaml::from_str(&config_content)
            .expect(&format!("failed to parse config file: {fname}"));

        // do validation
        ret.get_uri().expect("invalid proxy uri");

        info!("config loaded\n\t{ret:?}");
        ret
    }

    pub fn get_uri(&self) -> Result<Uri> {
        let uri_string = self.proxy.clone().expect("proxy is required");
        uri_string.try_into().map_err(|e: InvalidUri| anyhow!(e))
    }

    pub fn apply(&mut self, another: Self) {
        if another.enabled.is_some() {
            self.enabled = another.enabled;
        }
        if another.proxy.is_some() {
            self.proxy = another.proxy;
        }
        if another.status_code_threshold.is_some() {
            self.status_code_threshold = another.status_code_threshold;
        }
    }
}

pub async fn config_handler(Query(incomming): Query<Config>) -> Json<&'static Config> {
    unsafe {
        G_CONFIG.apply(incomming);
        return Json(&G_CONFIG);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_file() {
        let config = Config::from_file("config.yaml");
        println!("{:?}", config);
    }
}
