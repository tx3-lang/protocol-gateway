use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use tx3_sdk::trp::ClientOptions;

const PUBLIC_PREVIEW_TRP_KEY: &str = "trp1ffyf88ugcyg6j6n3yuh";
const PUBLIC_PREPROD_TRP_KEY: &str = "trp1mtg35n2n9lv7yauanfa";
const PUBLIC_MAINNET_TRP_KEY: &str = "trp1lrnhzcax5064cgxsaup";

pub struct Config {
    pub protocols_dir: PathBuf,
    pub port: u16,
    pub trp_override: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            trp_override: env::var("TRP_URL").ok(),
            protocols_dir: env::var("PROTOCOLS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./protocols")),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
        }
    }
}

pub fn trp_options_for_network(network: &str, trp_override: &Option<String>) -> Option<ClientOptions> {
    if let Some(url) = trp_override {
        return Some(ClientOptions {
            endpoint: url.clone(),
            headers: None,
        });
    }

    match network {
        "mainnet" => Some(ClientOptions {
            endpoint: "https://cardano-mainnet.trp-m1.demeter.run".into(),
            headers: Some(HashMap::from([(
                "dmtr-api-key".into(),
                PUBLIC_MAINNET_TRP_KEY.into(),
            )])),
        }),
        "preview" => Some(ClientOptions {
            endpoint: "https://cardano-preview.trp-m1.demeter.run".into(),
            headers: Some(HashMap::from([(
                "dmtr-api-key".into(),
                PUBLIC_PREVIEW_TRP_KEY.into(),
            )])),
        }),
        "preprod" => Some(ClientOptions {
            endpoint: "https://cardano-preprod.trp-m1.demeter.run".into(),
            headers: Some(HashMap::from([(
                "dmtr-api-key".into(),
                PUBLIC_PREPROD_TRP_KEY.into(),
            )])),
        }),
        _ => None,
    }
}
