use serde::Deserialize;

// ── Wire types (deserialised from JSON) ──────────────────────────────────────

/// Raw response from `GET deposit/accounts/me`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    pub reseller_id: Option<String>,
    pub reseller_name: Option<String>,
    pub usd_balance: Option<f64>,
    pub try_balance: Option<f64>,
}

// ── Public-facing types ───────────────────────────────────────────────────────

/// Full reseller profile including all currency balances.
#[derive(Debug)]
pub struct ResellerDetails {
    pub id: String,
    pub name: String,
    /// Always `true`; the API does not expose an active/inactive flag.
    pub active: bool,
    pub balances: Vec<Balance>,
}

/// A single currency balance entry.
#[derive(Debug, Clone)]
pub struct Balance {
    pub balance: f64,
    pub currency: String,
    pub symbol: String,
}

/// Normalised balance returned by [`crate::DnaClient::get_current_balance`].
#[derive(Debug)]
pub struct CurrentBalance {
    pub balance: f64,
    pub currency_id: u32,
    pub currency_name: String,
    pub currency_symbol: String,
}
