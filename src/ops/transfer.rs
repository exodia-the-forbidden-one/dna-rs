use serde_json::{Value, json};
use std::collections::HashMap;

use crate::client::DnaClient;
use crate::error::DnaResult;
use crate::models::contact::{ContactInput, ContactPayload};
use crate::models::domain::DomainInfo;
use crate::models::domain::DomainInfoResponse;
use crate::models::transfer::{
    DomainActionPayload, TransferCheckResponse, TransferCheckResult, TransferPayload,
    TransferStatusResponse,
};
use crate::ops::util::{build_contact_payload, parse_domain_info};

impl DnaClient {
    /// Initiate a domain transfer into the account.
    pub async fn transfer(
        &self,
        domain_name: &str,
        epp_code: &str,
        period: u32,
        contacts: Option<HashMap<&str, ContactInput>>,
    ) -> DnaResult<DomainInfo> {
        let payload_contacts: Vec<ContactPayload> = contacts
            .unwrap_or_default()
            .iter()
            .map(|(t, c)| build_contact_payload(c, t))
            .collect();

        let payload = TransferPayload {
            domain_name: domain_name.into(),
            auth_code: epp_code.into(),
            period,
            contacts: payload_contacts,
        };

        let raw: DomainInfoResponse = self.http.post("domains/transfer", &payload).await?;
        parse_domain_info(raw)
    }

    /// Check whether a domain can be transferred with the given auth code.
    pub async fn check_transfer(
        &self,
        domain_name: &str,
        auth_code: &str,
    ) -> DnaResult<TransferCheckResult> {
        let payload = json!({ "domainName": domain_name, "authCode": auth_code });
        let resp: TransferCheckResponse =
            self.http.post("domains/transfers/check", &payload).await?;

        let available = resp
            .transfer_availability_status
            .as_ref()
            .map(|v| !v.is_null() && v != &Value::Bool(false) && v != &json!(0) && v != &json!(""))
            .unwrap_or(false);

        Ok(TransferCheckResult {
            transfer_available: available,
            auth_code_is_required: resp.auth_code_is_required.unwrap_or(false),
            auth_code_is_valid: resp.auth_code_is_valid.unwrap_or(false),
            user_transfer_required: resp.user_transfer_required.unwrap_or(false),
            transfer_lock: resp.transfer_lock.unwrap_or(false),
            message: resp.message.unwrap_or_default(),
            message_key: resp.message_key.unwrap_or_default(),
        })
    }

    /// Cancel a pending incoming transfer.
    pub async fn cancel_transfer(&self, domain_name: &str) -> DnaResult<String> {
        let payload = DomainActionPayload {
            domain_name: domain_name.into(),
        };
        let resp: TransferStatusResponse =
            self.http.post("domains/transfers/cancel", &payload).await?;
        Ok(resp.status.unwrap_or_else(|| "Cancelled".into()))
    }

    /// Approve a pending outgoing transfer.
    pub async fn approve_transfer(&self, domain_name: &str) -> DnaResult<String> {
        let payload = DomainActionPayload {
            domain_name: domain_name.into(),
        };
        let resp: TransferStatusResponse = self
            .http
            .post("domains/transfers/approve", &payload)
            .await?;
        Ok(resp.status.unwrap_or_else(|| "Approved".into()))
    }

    /// Reject a pending outgoing transfer.
    pub async fn reject_transfer(&self, domain_name: &str) -> DnaResult<String> {
        let payload = DomainActionPayload {
            domain_name: domain_name.into(),
        };
        let resp: TransferStatusResponse =
            self.http.post("domains/transfers/reject", &payload).await?;
        Ok(resp.status.unwrap_or_else(|| "Rejected".into()))
    }
}
