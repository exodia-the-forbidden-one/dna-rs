use serde_json::Value;

use crate::client::DnaClient;
use crate::error::DnaResult;
use crate::models::contact::{ContactInput, ContactPayload};
use crate::models::domain::{
    DomainInfo, DomainInfoResponse, DomainList, DomainListItem, DomainSummary, LockPayload,
    PrivacyPayload, RegisterPayload, RenewPayload, RenewResponse, RenewResult,
};
use crate::ops::util::{build_contact_payload, parse_domain_info};
use std::collections::HashMap;

/// Default nameservers used when none are supplied by the caller.
const DEFAULT_NS: &[&str] = &["ns1.domainnameapi.com", "ns2.domainnameapi.com"];

impl DnaClient {
    // ── List ─────────────────────────────────────────────────────────────────

    /// List all domains in the account.
    ///
    /// Pass `extra` key/value pairs to override pagination defaults
    /// (`MaxResultCount` = 200, `SkipCount` = 0).
    pub async fn get_list(&self, extra: Option<&[(&str, &str)]>) -> DnaResult<DomainList> {
        let mut params: Vec<(&str, String)> =
            vec![("MaxResultCount", "200".into()), ("SkipCount", "0".into())];
        if let Some(overrides) = extra {
            for (k, v) in overrides {
                params.retain(|(pk, _)| pk != k);
                params.push((k, v.to_string()));
            }
        }

        let query: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let raw: Value = self.http.get("domains", Some(&query)).await?;

        let total_count = raw.get("totalCount").and_then(Value::as_u64).unwrap_or(0);

        let items: Vec<DomainListItem> = raw
            .get("items")
            .and_then(Value::as_array)
            .map(|arr| serde_json::from_value(Value::Array(arr.clone())))
            .transpose()?
            .unwrap_or_default();

        let domains = items
            .into_iter()
            .map(|item| DomainSummary {
                id: item.id.unwrap_or(0),
                status: item
                    .status_text
                    .or_else(|| {
                        item.status
                            .as_ref()
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    })
                    .unwrap_or_default(),
                domain_name: item.domain_name.unwrap_or_default(),
                auth_code: item.auth_code.unwrap_or_default(),
                lock_status: item.lock_status.unwrap_or(false),
                privacy_protection_status: item.privacy_protection_status.unwrap_or(false),
                is_child_name_server: item.hosts.as_ref().map(|h| !h.is_empty()).unwrap_or(false),
                name_servers: item.name_servers.unwrap_or_default(),
                start_date: item.start_date.unwrap_or_default(),
                expiration_date: item.expiration_date.unwrap_or_default(),
                remaining_days: item.remaining_day.unwrap_or(0),
            })
            .collect();

        Ok(DomainList {
            domains,
            total_count,
        })
    }

    // ── Detail ────────────────────────────────────────────────────────────────

    /// Fetch full details for a single domain.
    pub async fn get_details(&self, domain_name: &str) -> DnaResult<DomainInfo> {
        let query = [("DomainName", domain_name)];
        let raw: DomainInfoResponse = self.http.get("domains/info", Some(&query)).await?;
        parse_domain_info(raw)
    }

    /// Synchronise domain data from the registry (delegates to [`get_details`]).
    pub async fn sync_from_registry(&self, domain_name: &str) -> DnaResult<DomainInfo> {
        self.get_details(domain_name).await
    }

    // ── Renew ─────────────────────────────────────────────────────────────────

    /// Renew a domain for `period` additional years.
    pub async fn renew(&self, domain_name: &str, period: u32) -> DnaResult<RenewResult> {
        let payload = RenewPayload {
            domain_name: domain_name.into(),
            period,
        };
        let resp: RenewResponse = self.http.post("domains/renew", &payload).await?;

        let expiration_date = resp.expiration_date.ok_or_else(|| {
            crate::error::DnaError::UnexpectedResponse(
                "Renew response missing `expirationDate`".into(),
            )
        })?;

        Ok(RenewResult { expiration_date })
    }

    // ── Register ──────────────────────────────────────────────────────────────

    /// Register a new domain with full contact information.
    pub async fn register_with_contact_info(
        &self,
        domain_name: &str,
        period: u32,
        contacts: HashMap<&str, ContactInput>,
        name_servers: Option<Vec<String>>,
        epp_lock: bool,
        privacy_lock: bool,
        additional_attributes: Option<Value>,
    ) -> DnaResult<DomainInfo> {
        let payload_contacts: Vec<ContactPayload> = contacts
            .iter()
            .map(|(t, c)| build_contact_payload(c, t))
            .collect();

        let ns = name_servers.unwrap_or_else(|| DEFAULT_NS.iter().map(|s| s.to_string()).collect());

        let payload = RegisterPayload {
            domain_name: domain_name.into(),
            period,
            name_servers: ns,
            is_locked: epp_lock,
            privacy_enabled: privacy_lock,
            contacts: payload_contacts,
            additional_attributes: additional_attributes
                .unwrap_or_else(|| Value::Object(Default::default())),
        };

        let raw: DomainInfoResponse = self
            .http
            .post("domains/register-with-contacts", &payload)
            .await?;
        parse_domain_info(raw)
    }

    // ── Lock ──────────────────────────────────────────────────────────────────

    /// Enable the registrar transfer lock.
    pub async fn enable_theft_protection_lock(&self, domain_name: &str) -> DnaResult<()> {
        let payload = LockPayload {
            domain_name: domain_name.into(),
            lock_status: true,
        };
        let _: Value = self.http.post("domains/lock", &payload).await?;
        Ok(())
    }

    /// Disable the registrar transfer lock.
    pub async fn disable_theft_protection_lock(&self, domain_name: &str) -> DnaResult<()> {
        let payload = LockPayload {
            domain_name: domain_name.into(),
            lock_status: false,
        };
        let _: Value = self.http.post("domains/lock", &payload).await?;
        Ok(())
    }

    // ── Privacy ───────────────────────────────────────────────────────────────

    /// Enable or disable WHOIS privacy protection.
    ///
    /// `_reason` is accepted for API-compatibility but not forwarded (the REST
    /// gateway does not expose a reason field).
    pub async fn modify_privacy_protection_status(
        &self,
        domain_name: &str,
        status: bool,
        _reason: Option<&str>,
    ) -> DnaResult<bool> {
        let payload = PrivacyPayload {
            domain_name: domain_name.into(),
            privacy_status: status,
        };
        let _: Value = self.http.post("domains/privacy", &payload).await?;
        Ok(status)
    }

    // ── Utility ───────────────────────────────────────────────────────────────

    /// Returns `true` if the domain has a `.tr` TLD.
    pub fn is_tr_tld(&self, domain: &str) -> bool {
        domain.to_lowercase().ends_with(".tr")
    }

    /// Validate and normalise a [`ContactInput`], filling in Turkish defaults
    /// for any blank mandatory fields (mirrors `validateContact` in the PHP library).
    pub fn validate_contact(&self, mut c: ContactInput) -> ContactInput {
        fn fill(s: &mut String, default: &str) {
            if s.trim().is_empty() {
                *s = default.to_owned();
            }
        }

        // Capture first_name before borrowing mutably via fill
        let first = c.first_name.clone();

        fill(&mut c.first_name, "Isimyok");
        fill(
            &mut c.last_name,
            if first.is_empty() { "Isimyok" } else { &first },
        );
        fill(&mut c.address_line1, "Addres yok");
        fill(&mut c.city, "ISTANBUL");
        fill(&mut c.country, "TR");
        fill(&mut c.zip_code, "34000");
        fill(&mut c.phone, "5555555555");
        fill(&mut c.phone_country_code, "90");

        // Strip non-digits, then apply country-code rules.
        let digits: String = c.phone.chars().filter(|ch| ch.is_ascii_digit()).collect();
        match digits.len() {
            10 => {
                c.phone_country_code = String::new();
                c.phone = digits;
            }
            11 if digits.starts_with('9') => {
                c.phone_country_code = digits[..2].to_owned();
                c.phone = digits[2..].to_owned();
            }
            12 if digits.starts_with("90") => {
                c.phone_country_code = digits[..2].to_owned();
                c.phone = digits[2..].to_owned();
            }
            _ => {
                c.phone_country_code = "90".to_owned();
                c.phone = if digits.is_empty() {
                    "5555555555".into()
                } else {
                    digits
                };
            }
        }

        fill(&mut c.phone_country_code, "90");
        fill(&mut c.phone, "5555555555");
        c
    }
}
