use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Wire type ─────────────────────────────────────────────────────────────────

/// Raw contact as returned by the gateway (many optional aliases for field names).
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContactRaw {
    pub handle: Option<String>,
    pub id: Option<Value>,
    pub contact_type: Option<String>,
    pub status: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company_name: Option<String>,
    pub organization_name: Option<String>,
    pub e_mail: Option<String>,
    pub email_address: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub address_line3: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub state_or_province: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub postal_code: Option<String>,
    pub zip_code: Option<String>,
    pub phone: Option<String>,
    pub phone_number: Option<String>,
    pub phone_country_code: Option<String>,
    pub fax: Option<String>,
    pub fax_number: Option<String>,
    pub fax_country_code: Option<String>,
    pub additional_attributes: Option<Value>,
}

// ── Public-facing types ───────────────────────────────────────────────────────

/// Normalised contact returned to callers.
#[derive(Debug, Clone)]
pub struct ContactInfo {
    pub id: String,
    pub status: String,
    pub first_name: String,
    pub last_name: String,
    pub company: String,
    pub email: String,
    pub contact_type: String,
    pub address: ContactAddress,
    pub phone: ContactPhone,
    pub additional: Value,
}

#[derive(Debug, Clone, Default)]
pub struct ContactAddress {
    pub line1: String,
    pub line2: String,
    pub line3: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub zip_code: String,
}

#[derive(Debug, Clone, Default)]
pub struct ContactPhone {
    pub number: String,
    pub country_code: String,
    pub fax: String,
    pub fax_cc: String,
}

/// Caller-supplied contact data used for registration, transfer, or update.
#[derive(Debug, Clone, Default)]
pub struct ContactInput {
    pub first_name: String,
    pub last_name: String,
    pub company: String,
    pub email: String,
    pub address_line1: String,
    pub address_line2: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub zip_code: String,
    pub phone: String,
    pub phone_country_code: String,
    pub fax: String,
    pub fax_country_code: String,
}

// ── Wire payload ──────────────────────────────────────────────────────────────

/// Serialised contact sent to the API.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactPayload {
    pub contact_type: String,
    pub first_name: String,
    pub last_name: String,
    pub company_name: String,
    pub e_mail: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub postal_code: String,
    pub phone_country_code: String,
    pub phone: String,
    pub fax_country_code: String,
    pub fax: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveContactsPayload {
    pub domain_name: String,
    pub contacts: Vec<ContactPayload>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveContactsResponse {
    pub contacts: Option<Vec<ContactRaw>>,
}

// ── Helper ────────────────────────────────────────────────────────────────────

/// Map a caller-facing contact type label to the API's type string.
pub fn contact_type_to_api(t: &str) -> &'static str {
    match t.to_lowercase().as_str() {
        "administrative" | "admin" => "Admin",
        "technical" | "tech" => "Tech",
        "billing" => "Billing",
        "registrant" => "Registrant",
        _ => "Registrant",
    }
}
