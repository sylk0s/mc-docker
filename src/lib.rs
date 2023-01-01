use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use server::Server;
use cloud::CloudSync;
use std::fs;
use serde::Deserialize;
use std::process::Command;

pub mod server;
pub mod net;
pub mod status;
pub mod cloud;
pub mod handlers;

pub type Servers = Arc<RwLock<HashMap<String, Server>>>;

pub async fn run() {
    let config = Config::get();

    let servers = load_from_cloud().await.unwrap();

    println!("Servers: {:?}", servers.write().await);

    load_modules(config.clone()).await;
    net::start_ws(servers, config).await;    
}

pub async fn load_from_cloud() -> Option<Servers> {
    match Server::clget().await {
        Ok(cl_servers) => {
            let mut servers = HashMap::new();
            for server in cl_servers {
                servers.insert(server.name.clone(), server); 
            }
            Some(Arc::new(RwLock::new(servers)))
        },
        Err(_) => None,
    }
}

// change to use the $HOME
pub const CONF_PATH: &str = "/home/sylkos/.config/mc-docker";

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub fb_id: String,
    pub ws_port: u16,
    pub path: String,
    pub modules: Vec<String>,
}

impl Config {
    fn get() -> Config {
        let conf_file = fs::read_to_string(format!("{CONF_PATH}/config.toml")).expect("Failed to read config from fs");
        toml::from_str(&conf_file).expect("Error parsing config file from toml")
    }
}

async fn load_modules(config: Config) {
    for module in config.modules {
        tokio::spawn(async move {
            // change this so we can specific a module path?
            let _m = Command::new(module)
                .spawn().unwrap();
        });
    }
}
