//! Types are grouped by domain:
//! - [`account`]     – reseller details, balances
//! - [`domain`]      – domain list, info, renew, register
//! - [`contact`]     – contact CRUD models
//! - [`tld`]         – TLD list and pricing
//! - [`transfer`]    – transfer initiation and checks

pub mod account;
pub mod contact;
pub mod domain;
pub mod tld;
pub mod transfer;

// Flatten everything so callers can just `use crate::models::*`
pub use account::*;
pub use contact::*;
pub use domain::*;
pub use tld::*;
pub use transfer::*;

use serde::Deserialize;

/// Generic paginated envelope returned by list endpoints.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total_count: u64,
}
