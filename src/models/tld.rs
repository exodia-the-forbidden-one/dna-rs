use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

// ── Wire type ─────────────────────────────────────────────────────────────────

/// Raw TLD entry from `GET products/tlds`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TldItem {
    pub name: Option<String>,
    pub status: Option<String>,
    pub min_registration_period: Option<u32>,
    pub max_registration_period: Option<u32>,
    pub constraints: Option<TldConstraints>,
    pub prices: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TldConstraints {
    /// Note: the upstream JSON field is `maxLenght` (sic).
    #[serde(alias = "maxLenght")]
    pub max_length: Option<u32>,
    pub min_length: Option<u32>,
}

// ── Public-facing types ───────────────────────────────────────────────────────

/// Price map keyed by registration period (years).
pub type PeriodPriceMap = BTreeMap<u32, f64>;

/// Normalised TLD info returned by [`crate::DnaClient::get_tld_list`].
#[derive(Debug)]
pub struct TldInfo {
    pub id: u32,
    pub tld: String,
    pub status: String,
    pub min_char: u32,
    pub max_char: u32,
    pub min_period: u32,
    pub max_period: u32,
    /// Pricing tables for each operation type (`registration`, `renew`, etc.)
    pub pricing: HashMap<String, PeriodPriceMap>,
    /// Currency code per operation type.
    pub currencies: HashMap<String, String>,
}
