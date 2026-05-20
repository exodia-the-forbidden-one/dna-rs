//! [`DnaClient`] struct and constructor methods.
//!
//! All API operation methods live in the `ops` sub-modules and are added
//! to this type via separate `impl DnaClient` blocks.

use crate::error::DnaResult;
use crate::http::HttpClient;
use crate::{URL_OTE, URL_PROD};

/// Async REST client for the Domain Name API.
///
/// Create one instance per reseller session and reuse it across calls.
///
/// # Examples
///
/// ```rust,no_run
/// use dna_rs::DnaClient;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), dna_rs::DnaError> {
/// // Production
/// let client = DnaClient::new("RESELLER-UUID", "API-TOKEN")?;
///
/// // OTE / test environment
/// let ote = DnaClient::new_ote("RESELLER-UUID", "API-TOKEN")?;
/// # Ok(())
/// # }
/// ```
pub struct DnaClient {
    pub(crate) http: HttpClient,
}

impl DnaClient {
    /// Connect to the **production** endpoint.
    pub fn new(reseller_id: &str, token: &str) -> DnaResult<Self> {
        Ok(Self {
            http: HttpClient::new(URL_PROD, reseller_id, token)?,
        })
    }

    /// Connect to the **OTE** (test/sandbox) endpoint.
    pub fn new_ote(reseller_id: &str, token: &str) -> DnaResult<Self> {
        Ok(Self {
            http: HttpClient::new(URL_OTE, reseller_id, token)?,
        })
    }

    /// Connect to a **custom** base URL (useful for mocking in tests).
    pub fn with_url(reseller_id: &str, token: &str, url: &str) -> DnaResult<Self> {
        Ok(Self {
            http: HttpClient::new(url, reseller_id, token)?,
        })
    }
}
