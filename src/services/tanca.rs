use std::sync::OnceLock;

use super::client::{Client, Config, DEFAULT_BASE_URL};
use crate::logf;

static CLIENT: OnceLock<Client> = OnceLock::new();

/// Returns the process-wide Tanca client, built from the environment on
/// first use. Exits the process when `TANCA_TOKEN` is not set.
pub fn tanca_client() -> &'static Client {
    CLIENT.get_or_init(|| Client::new(load_tanca_config()))
}

/// Reads the Tanca configuration from environment variables.
pub fn load_tanca_config() -> Config {
    let var = |k: &str| std::env::var(k).unwrap_or_default();
    let token = var("TANCA_TOKEN");
    if token.is_empty() {
        logf!("TANCA_TOKEN is required");
        std::process::exit(1);
    }
    Config {
        base_url: DEFAULT_BASE_URL.into(),
        token,
        // Optional; fetched from the API when not set.
        shop_id: var("TANCA_SHOP_ID"),
        branch_id: var("TANCA_BRANCH_ID"),
        device_header: var("TANCA_DEVICE"),
        timezone: "Asia/Saigon".into(),
        lang: "vi".into(),
        is_admin: "1".into(),
    }
}
