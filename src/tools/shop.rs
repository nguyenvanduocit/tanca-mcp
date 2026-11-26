use serde_json::Value;

use super::{ToolEntry, ToolError, entry, failed};
use crate::format;
use crate::services::tanca_client;

pub(super) fn tools() -> Vec<ToolEntry> {
    vec![entry(
        "tanca_get_shop_info",
        "Get company/shop information from Tanca. Returns shop ID, name, address, contact info, subscription packages, and other company details. Useful for getting shop_id without needing to configure it manually.",
        &[],
        |a| Box::pin(get_shop_info(a)),
    )]
}

async fn get_shop_info(_args: Value) -> Result<String, ToolError> {
    let shop = tanca_client()
        .get_shop_detail()
        .await
        .map_err(|e| failed("failed to get shop info", e))?;
    Ok(format::shop(&shop))
}
