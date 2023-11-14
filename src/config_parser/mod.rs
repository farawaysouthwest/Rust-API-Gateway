use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};
use log::{debug, info};
use serde_yaml;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    pub path: String,
    pub target_service: String,
    pub target_port: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GatewayConfig {
    pub authorization_api_url: String,
    pub gateway_port: String,
    pub services: Vec<ServiceConfig>,
}

pub fn load_config(path: &str) -> GatewayConfig {

    info!("Loading config from {}", path);

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => panic!("Unable to open config file")
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read config file");


    let config: GatewayConfig = serde_yaml::from_str(&contents).expect("Unable to parse config file");

    debug!("Config loaded: {:?}", config);
    config
}