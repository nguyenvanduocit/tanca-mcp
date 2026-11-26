use anyhow::{Context, Result};

use super::client::{Client, decode};
use super::types::Shop;

impl Client {
    /// Retrieves the current shop/company information.
    pub async fn get_shop_detail(&self) -> Result<Shop> {
        let body = self
            .get("/shop/detail", None)
            .await
            .context("get shop detail")?;
        decode(&body)
    }
}
