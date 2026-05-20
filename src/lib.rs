//! # dna-rs
//!
//! Async Rust client for the [Domain Name API](https://www.domainnameapi.com/) REST gateway.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use dna_rs::DnaClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), dna_rs::DnaError> {
//!     let client = DnaClient::new("YOUR-RESELLER-UUID", "YOUR-API-TOKEN")?;
//!     let balance = client.get_current_balance("USD").await?;
//!     println!("Balance: {} {}", balance.balance, balance.currency_name);
//!     Ok(())
//! }
//! ```
//!
//! ## Module layout
//!
//! | Path | Contents |
//! |---|---|
//! | [`models`] | All request / response types, grouped by domain |
//! | `ops/` (private) | `impl DnaClient` blocks, one file per API domain |
//! | [`error`] | [`DnaError`] and [`DnaResult`] |
//! | `http` (private) | Low-level reqwest transport |
//!
//!
//! ## Environments
//!
//! | Constructor | Endpoint |
//! |---|---|
//! | [`DnaClient::new`] | Production |
//! | [`DnaClient::new_ote`] | OTE / sandbox |
//! | [`DnaClient::with_url`] | Custom URL |
//!
//! ## Error Handling
//!
//! All methods return [`DnaResult<T>`] which is [`Result<T, DnaError>`].
//!
//! ```rust,no_run
//! use dna_rs::{DnaClient, DnaError};
//!
//! # async fn example() -> Result<(), DnaError> {
//! # let client = DnaClient::new("id", "token")?;
//! match client.get_reseller_details().await {
//!     Ok(d)                                    => println!("{}", d.name),
//!     Err(DnaError::Api { code, message, .. }) => eprintln!("[{code}] {message}"),
//!     Err(e)                                   => eprintln!("{e}"),
//! }
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod models;

mod client;
mod http;
mod ops;

pub use client::DnaClient;
pub use error::{DnaError, DnaResult};

/// Production API base URL.
pub const URL_PROD: &str = "https://api.domainresellerapi.com/api/v1";
/// OTE (test/sandbox) API base URL.
pub const URL_OTE: &str = "https://ote.domainresellerapi.com/api/v1";
