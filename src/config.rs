use hyper::StatusCode;
use serde::Deserialize;
use std::fs;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub proxy: Proxy,
    #[serde(with = "crate::serde_cache::status_code")]
    pub status_code_threshold: StatusCode
}

#[derive(Clone, Debug, Deserialize)]
pub struct Proxy {
    pub authority: String,
    pub scheme: String,
}

impl Config {
    pub fn from_file(fname: &str) -> Self {
        let config_content =
            fs::read_to_string(fname).expect(&format!("failed to read from file: {fname}"));

        serde_yaml::from_str(&config_content)
            .expect(&format!("failed to parse config file: {fname}"))
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
