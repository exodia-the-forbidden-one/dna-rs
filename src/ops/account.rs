use serde_json::Value;

use crate::client::DnaClient;
use crate::error::{DnaError, DnaResult};
use crate::models::account::{AccountResponse, Balance, CurrentBalance, ResellerDetails};
use crate::ops::util::currency_meta;

impl DnaClient {
    /// Fetch reseller profile including all currency balances.
    pub async fn get_reseller_details(&self) -> DnaResult<ResellerDetails> {
        let raw: AccountResponse = self.http.get("deposit/accounts/me", None::<&()>).await?;

        let id = raw
            .reseller_id
            .ok_or_else(|| DnaError::UnexpectedResponse("Response missing `resellerId`".into()))?;

        Ok(ResellerDetails {
            id,
            name: raw.reseller_name.unwrap_or_default(),
            active: true, // API does not expose an active flag
            balances: vec![
                Balance {
                    balance: raw.usd_balance.unwrap_or(0.0),
                    currency: "USD".into(),
                    symbol: "$".into(),
                },
                Balance {
                    balance: raw.try_balance.unwrap_or(0.0),
                    currency: "TL".into(),
                    symbol: "TL".into(),
                },
            ],
        })
    }

    /// Fetch the balance for a single currency (`"USD"`, `"TRY"`, `"EUR"`, `"GBP"`).
    pub async fn get_current_balance(&self, currency_id: &str) -> DnaResult<CurrentBalance> {
        let currency_upper = currency_id.to_uppercase();
        let balance_key = format!("{}Balance", currency_id.to_lowercase());
        let query = [("currency", currency_upper.as_str())];

        // Deserialise as Value so we can handle any currency key dynamically.
        let raw: Value = self.http.get("deposit/accounts/me", Some(&query)).await?;
        let balance = raw.get(&balance_key).and_then(Value::as_f64).unwrap_or(0.0);

        let (cur_id, cur_name, cur_symbol) = currency_meta(&currency_upper);

        Ok(CurrentBalance {
            balance,
            currency_id: cur_id,
            currency_name: cur_name.into(),
            currency_symbol: cur_symbol.into(),
        })
    }
}
