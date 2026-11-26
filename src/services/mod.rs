//! Tanca API client and endpoint wrappers.

mod client;
mod employees;
mod requests;
mod shifts;
mod shop;
mod tanca;
pub mod types;

pub use client::{ApiError, Client, Config, DEFAULT_BASE_URL, Query, encode_query};
pub use employees::GetEmployeeListParams;
pub use shifts::GetEmployeeShiftSummaryParams;
pub use tanca::{load_tanca_config, tanca_client};
pub use types::*;
