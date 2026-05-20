use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::contact::ContactRaw;

// ── Availability ─────────────────────────────────────────────────────────────

/// Single query item sent to the bulk-search endpoint.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkSearchQuery {
    pub domain_name: String,
}

/// One item returned by the bulk-search endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkSearchItem {
    pub domain_name: Option<String>,
    pub tld: Option<String>,
    pub status: Option<String>,
    pub period: Option<u32>,
    pub is_premium: Option<bool>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub reason: Option<String>,
}

/// Availability result for one domain + TLD combination.
#[derive(Debug)]
pub struct AvailabilityResult {
    pub tld: String,
    pub domain_name: String,
    /// `"available"` or `"notavailable"`
    pub status: String,
    pub command: String,
    pub period: u32,
    pub is_fee: bool,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub reason: Option<String>,
}

// ── Domain list ───────────────────────────────────────────────────────────────

/// Wire type for one entry from `GET domains`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainListItem {
    pub id: Option<u64>,
    pub status: Option<Value>,
    pub status_text: Option<String>,
    pub domain_name: Option<String>,
    pub auth_code: Option<String>,
    pub lock_status: Option<bool>,
    pub privacy_protection_status: Option<bool>,
    pub hosts: Option<Vec<Value>>,
    pub name_servers: Option<Vec<String>>,
    pub start_date: Option<String>,
    pub expiration_date: Option<String>,
    pub remaining_day: Option<i64>,
}

/// Normalised domain entry returned in list results.
#[derive(Debug)]
pub struct DomainSummary {
    pub id: u64,
    pub status: String,
    pub domain_name: String,
    pub auth_code: String,
    pub lock_status: bool,
    pub privacy_protection_status: bool,
    pub is_child_name_server: bool,
    pub name_servers: Vec<String>,
    pub start_date: String,
    pub expiration_date: String,
    pub remaining_days: i64,
}

/// Return value of [`crate::DnaClient::get_list`].
#[derive(Debug)]
pub struct DomainList {
    pub domains: Vec<DomainSummary>,
    pub total_count: u64,
}

// ── Domain detail ─────────────────────────────────────────────────────────────

/// Wire response from `GET domains/info`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainInfoResponse {
    pub id: Option<u64>,
    pub status: Option<Value>,
    pub domain_name: Option<String>,
    pub name: Option<String>,
    pub auth_code: Option<String>,
    pub epp_code: Option<String>,
    pub lock_status: Option<bool>,
    pub privacy_protection_status: Option<bool>,
    pub hosts: Option<Vec<ChildNsRaw>>,
    pub nameservers: Option<Vec<String>>,
    pub contacts: Option<Vec<ContactRaw>>,
    pub start_date: Option<String>,
    pub expiration_date: Option<String>,
    pub remaining_day: Option<i64>,
    pub additional_attributes: Option<Value>,
}

/// Wire type for a glue (child) nameserver entry.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildNsRaw {
    pub name: Option<String>,
    pub ip_addresses: Option<Vec<IpEntry>>,
}

/// IP address entry with version tag.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IpEntry {
    pub ip_address: String,
    pub ip_version: String,
}

/// Fully normalised domain detail returned to callers.
#[derive(Debug)]
pub struct DomainInfo {
    pub id: u64,
    pub status: String,
    pub domain_name: String,
    pub auth_code: String,
    pub lock_status: bool,
    pub privacy_protection_status: bool,
    pub is_child_name_server: bool,
    pub name_servers: Vec<String>,
    pub contacts: ContactIds,
    pub start_date: String,
    pub expiration_date: String,
    pub remaining_days: i64,
    pub additional: Value,
    pub child_name_servers: Vec<ChildNs>,
}

/// Contact handle IDs keyed by role.
#[derive(Debug, Default)]
pub struct ContactIds {
    pub billing: String,
    pub technical: String,
    pub administrative: String,
    pub registrant: String,
}

/// Normalised child nameserver with a single representative IP.
#[derive(Debug)]
pub struct ChildNs {
    pub ns: String,
    pub ip: String,
}

// ── Nameserver payloads ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModifyNsPayload {
    pub domain_name: String,
    pub name_servers: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChildNsPayload {
    pub domain_name: String,
    pub host_name: String,
    pub ip_addresses: Vec<IpEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModifyChildNsPayload {
    pub domain_name: String,
    pub host_name: String,
    pub new_host_name: String,
    pub ip_addresses: Vec<IpEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsResponse {
    pub name_servers: Option<Vec<String>>,
    pub host_name: Option<String>,
    pub ip_addresses: Option<Vec<IpEntry>>,
}

// ── Lock / Privacy payloads ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LockPayload {
    pub domain_name: String,
    pub lock_status: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrivacyPayload {
    pub domain_name: String,
    pub privacy_status: bool,
}

// ── Renew / Register payloads ─────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenewPayload {
    pub domain_name: String,
    pub period: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewResponse {
    pub expiration_date: Option<String>,
}

/// Result of a successful renewal.
#[derive(Debug)]
pub struct RenewResult {
    pub expiration_date: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RegisterPayload {
    pub domain_name: String,
    pub period: u32,
    pub name_servers: Vec<String>,
    pub is_locked: bool,
    pub privacy_enabled: bool,
    pub contacts: Vec<super::contact::ContactPayload>,
    pub additional_attributes: Value,
}
