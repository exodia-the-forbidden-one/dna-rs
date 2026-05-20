use serde_json::Value;

use crate::client::DnaClient;
use crate::error::{DnaError, DnaResult};
use crate::models::domain::{AvailabilityResult, BulkSearchItem, BulkSearchQuery};

impl DnaClient {
    /// Check availability of all `domain × extension` combinations.
    ///
    /// `command` is forwarded verbatim (e.g. `"create"`, `"transfer"`).
    pub async fn check_availability(
        &self,
        domains: &[&str],
        extensions: &[&str],
        period: u32,
        command: &str,
    ) -> DnaResult<Vec<AvailabilityResult>> {
        let queries: Vec<BulkSearchQuery> = domains
            .iter()
            .flat_map(|d| {
                extensions.iter().map(move |ext| BulkSearchQuery {
                    domain_name: format!("{}.{}", d, ext.trim_start_matches('.')),
                })
            })
            .collect();

        if queries.is_empty() {
            return Err(DnaError::InvalidArgument("No domain names provided".into()));
        }

        let raw: Value = self.http.post("domains/bulk-search", &queries).await?;

        let items: Vec<BulkSearchItem> =
            if let Some(arr) = raw.get("infos").and_then(Value::as_array) {
                serde_json::from_value(Value::Array(arr.clone()))?
            } else if raw.is_array() {
                serde_json::from_value(raw)?
            } else {
                return Err(DnaError::UnexpectedResponse(
                    "Unexpected bulk-search response shape".into(),
                ));
            };

        let results = items
            .into_iter()
            .map(|item| {
                let domain_raw = item.domain_name.unwrap_or_default();
                let tld = item.tld.unwrap_or_else(|| {
                    domain_raw
                        .splitn(2, '.')
                        .nth(1)
                        .unwrap_or("")
                        .to_lowercase()
                });
                let domain_name = domain_raw
                    .to_lowercase()
                    .strip_suffix(&format!(".{}", tld))
                    .unwrap_or(domain_raw.to_lowercase().as_str())
                    .to_owned();

                let status_raw = item.status.unwrap_or_default().to_lowercase();
                let is_available = matches!(status_raw.as_str(), "available" | "1" | "true");

                AvailabilityResult {
                    tld,
                    domain_name,
                    status: if is_available {
                        "available".into()
                    } else {
                        "notavailable".into()
                    },
                    command: command.to_owned(),
                    period: item.period.unwrap_or(period),
                    is_fee: item.is_premium.unwrap_or(false),
                    price: item.price,
                    currency: item.currency,
                    reason: item.reason,
                }
            })
            .collect();

        Ok(results)
    }
}
