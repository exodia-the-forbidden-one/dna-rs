//! Low-level HTTP helpers used by the client.
//!
//! Handles:
//!   - building the `reqwest` client with the right headers
//!   - appending query params for GET/DELETE
//!   - decoding JSON success bodies
//!   - mapping non-2xx responses to [`DnaError::Api`]

use reqwest::{Client, Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::{DnaError, DnaResult};

/// Thin wrapper that holds the base URL + authentication state.
pub(crate) struct HttpClient {
    inner: Client,
    base_url: String,
    token: String,
    reseller_id: String,
}

impl HttpClient {
    pub fn new(base_url: &str, reseller_id: &str, token: &str) -> DnaResult<Self> {
        let inner = Client::builder()
            // Accept invalid certs to mirror PHP behaviour in dev; consider
            // making this configurable for production hardening.
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(DnaError::Http)?;

        Ok(Self {
            inner,
            base_url: base_url.to_string(),
            token: token.to_string(),
            reseller_id: reseller_id.to_string(),
        })
    }

    fn url(&self, endpoint: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        )
    }

    /// GET with optional query parameters.
    pub async fn get<Q, R>(&self, endpoint: &str, query: Option<&Q>) -> DnaResult<R>
    where
        Q: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let mut req = self
            .inner
            .request(Method::GET, self.url(endpoint))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-API-KEY", &self.token)
            .header("__reseller", &self.reseller_id);

        if let Some(q) = query {
            req = req.query(q);
        }

        self.execute(req).await
    }

    /// POST with a JSON body.
    pub async fn post<B, R>(&self, endpoint: &str, body: &B) -> DnaResult<R>
    where
        B: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let req = self
            .inner
            .request(Method::POST, self.url(endpoint))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-API-KEY", &self.token)
            .header("__reseller", &self.reseller_id)
            .json(body);

        self.execute(req).await
    }

    /// PUT with a JSON body.
    pub async fn put<B, R>(&self, endpoint: &str, body: &B) -> DnaResult<R>
    where
        B: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let req = self
            .inner
            .request(Method::PUT, self.url(endpoint))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-API-KEY", &self.token)
            .header("__reseller", &self.reseller_id)
            .json(body);

        self.execute(req).await
    }

    /// DELETE with an optional JSON body (some endpoints need it).
    pub async fn delete<B, R>(&self, endpoint: &str, body: Option<&B>) -> DnaResult<R>
    where
        B: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let mut req = self
            .inner
            .request(Method::DELETE, self.url(endpoint))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-API-KEY", &self.token)
            .header("__reseller", &self.reseller_id);

        if let Some(b) = body {
            req = req.json(b);
        }

        self.execute(req).await
    }

    /// Send the request and map the response.
    async fn execute<R>(&self, req: reqwest::RequestBuilder) -> DnaResult<R>
    where
        R: DeserializeOwned,
    {
        let response = req.send().await?;
        let status = response.status();

        // Map redirect-style auth failures the same way the PHP code does.
        if status == StatusCode::MOVED_PERMANENTLY || status == StatusCode::FOUND {
            return Err(DnaError::Api {
                code: "CREDENTIALS".into(),
                message: "Invalid API credentials".into(),
                details: "Authentication failed. Check your API key and reseller ID.".into(),
            });
        }

        let body_bytes = response.bytes().await?;

        if status.is_success() {
            let parsed: R = serde_json::from_slice(&body_bytes)?;
            return Ok(parsed);
        }

        // Try to extract a structured error message from the body.
        let json_body: Option<Value> = serde_json::from_slice(&body_bytes).ok();

        let (code, message, details) = extract_error_fields(json_body, status);
        Err(DnaError::Api {
            code,
            message,
            details,
        })
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

fn extract_error_fields(body: Option<Value>, status: StatusCode) -> (String, String, String) {
    let fallback_code = format!("HTTP_{}", status.as_u16());

    let Some(v) = body else {
        return (fallback_code, "Empty error body".into(), String::new());
    };

    // Flat shape: { "message": "...", "code": "...", "details": "..." }
    // Nested shape: { "error": { "message": "...", "code": "...", "details": "..." } }
    let obj = if v.get("error").is_some() {
        &v["error"]
    } else {
        &v
    };

    let code = string_field(obj, "code").unwrap_or(fallback_code);
    let message = string_field(obj, "message").unwrap_or_else(|| "Unknown error".into());
    let details = string_field(obj, "details").unwrap_or_default();

    (code, message, details)
}

fn string_field(v: &Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().map(str::to_owned)
}
