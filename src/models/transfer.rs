use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::contact::ContactPayload;

// ── Wire types ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferPayload {
    pub domain_name: String,
    pub auth_code: String,
    pub period: u32,
    pub contacts: Vec<ContactPayload>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DomainActionPayload {
    pub domain_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferStatusResponse {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferCheckResponse {
    pub transfer_availability_status: Option<Value>,
    pub auth_code_is_required: Option<bool>,
    pub auth_code_is_valid: Option<bool>,
    pub user_transfer_required: Option<bool>,
    pub transfer_lock: Option<bool>,
    pub message: Option<String>,
    pub message_key: Option<String>,
}

// ── Public-facing types ───────────────────────────────────────────────────────

/// Result of a transfer-check call.
#[derive(Debug)]
pub struct TransferCheckResult {
    pub transfer_available: bool,
    pub auth_code_is_required: bool,
    pub auth_code_is_valid: bool,
    pub user_transfer_required: bool,
    pub transfer_lock: bool,
    pub message: String,
    pub message_key: String,
}
