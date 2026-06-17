use crate::client::DnaClient;
use crate::error::DnaResult;
use crate::models::TldListResponse;
use crate::models::tld::{TldInfo, TldItem};
use crate::ops::util::parse_tld_pricing;

impl DnaClient {
    /// Fetch TLD list and pricing matrix.
    pub async fn get_tld_list(
        &self,
        result_count: u32,
        skip_count: u32,
    ) -> DnaResult<TldListResponse> {
        let query = [
            ("MaxResultCount", result_count.to_string()),
            ("SkipCount", skip_count.to_string()),
        ];
        let raw: serde_json::Value = self.http.get("products/tlds", Some(&query)).await?;

        let total_count = raw
            .get("totalCount")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as u32;

        let items = raw
            .get("items")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut result = Vec::with_capacity(items.len());

        for (idx, tld_val) in items.into_iter().enumerate() {
            let tld: TldItem = serde_json::from_value(tld_val)?;

            let min_char = tld
                .constraints
                .as_ref()
                .and_then(|c| c.min_length)
                .unwrap_or(1);
            let max_char = tld
                .constraints
                .as_ref()
                .and_then(|c| c.max_length)
                .unwrap_or(63);
            let (pricing, currencies) = parse_tld_pricing(&tld.prices);

            result.push(TldInfo {
                id: (idx + 1) as u32,
                tld: tld.name.unwrap_or_default(),
                status: tld.status.unwrap_or_else(|| "Active".into()),
                min_char,
                max_char,
                min_period: tld.min_registration_period.unwrap_or(1),
                max_period: tld.max_registration_period.unwrap_or(10),
                pricing,
                currencies,
            });
        }

        Ok(TldListResponse {
            tld_items: result,
            total_count,
        })
    }
}
