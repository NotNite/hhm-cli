use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub prefix: String,
    pub api_key: String,
    pub ssh_keys: Vec<String>,

    pub image: String,
    pub instance_type: String,
    pub zone: String,

    pub cloud_init: String,

    pub labels: HashMap<String, String>,
}
