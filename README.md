# dna-rs

An async Rust client library for the [Domain Name API](https://www.domainnameapi.com/) REST gateway.

---

## Features

- Full coverage of the Domain Name API REST surface
  - Account & balance queries
  - Domain availability (bulk search)
  - Domain list, detail, registration, renewal
  - Contact management
  - Name-server and glue record management
  - Transfer initiation, approval, rejection, and cancellation
  - Lock and privacy protection toggles
  - TLD list with pricing matrices
- Production and OTE (sandbox) environments with a single constructor switch
- Structured errors — every failure is a typed `DnaError` variant
- 113 tests covering success paths, error paths, and edge cases

---

## Requirements

| Dependency | Version        |
| ---------- | -------------- |
| Rust       | 1.75+ (stable) |
| Tokio      | 1.x            |
| reqwest    | 0.12           |

---

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
dna-rs    = { version = "0.1.0 }
tokio     = { version = "1", features = ["rt-multi-thread", "macros"] }
```

---

## Quick Start

```rust
use dna_rs::DnaClient;

#[tokio::main]
async fn main() -> Result<(), dna_rs::DnaError> {
    let client = DnaClient::new("YOUR-RESELLER-UUID", "YOUR-API-TOKEN")?;

    // Check your account balance
    let balance = client.get_current_balance("USD").await?;
    println!("Balance: {} {}", balance.balance, balance.currency_name);

    // Check domain availability
    let results = client
        .check_availability(&["example"], &["com", "net", "io"], 1, "create")
        .await?;

    for r in &results {
        println!("{}.{} → {}", r.domain_name, r.tld, r.status);
    }

    Ok(())
}
```

---

## Environments

```rust
// Production (default)
let client = DnaClient::new("RESELLER-UUID", "API-TOKEN")?;

// OTE / sandbox
let client = DnaClient::new_ote("RESELLER-UUID", "API-TOKEN")?;

// Custom URL (useful for testing or proxies)
let client = DnaClient::with_url("RESELLER-UUID", "API-TOKEN", "https://my-proxy/api/v1")?;
```

---

## Error Handling

All methods return `DnaResult<T>`, which is `Result<T, DnaError>`.

```rust
use dna_rs::DnaError;

match client.get_reseller_details().await {
    Ok(details) => println!("Reseller: {}", details.name),
    Err(DnaError::Api { code, message, details }) => {
        eprintln!("API rejected the request [{code}]: {message}");
        eprintln!("Details: {details}");
    }
    Err(DnaError::Http(e))              => eprintln!("Network error: {e}"),
    Err(DnaError::Deserialize(e))       => eprintln!("Bad response JSON: {e}"),
    Err(DnaError::UnexpectedResponse(m)) => eprintln!("Unexpected shape: {m}"),
    Err(DnaError::InvalidArgument(m))   => eprintln!("Bad argument: {m}"),
}
```

| Variant                          | When                                                          |
| -------------------------------- | ------------------------------------------------------------- |
| `Api { code, message, details }` | The gateway returned a non-2xx status                         |
| `Http(reqwest::Error)`           | Network / TLS / timeout failure                               |
| `Deserialize(serde_json::Error)` | Response body is not valid JSON or has an unexpected shape    |
| `UnexpectedResponse(String)`     | A required field was missing from an otherwise valid response |
| `InvalidArgument(String)`        | The caller passed an empty or logically invalid argument      |

---

## Module Layout

```
src/
├── lib.rs               # Crate root, public re-exports
├── error.rs             # DnaError + DnaResult
├── client.rs            # DnaClient struct + constructors
├── http.rs              # reqwest transport (internal)
├── models/              # All request/response types
│   ├── account.rs       # Balance, ResellerDetails, CurrentBalance
│   ├── contact.rs       # ContactInfo, ContactInput, ContactPayload
│   ├── domain.rs        # DomainInfo, DomainList, AvailabilityResult, …
│   ├── tld.rs           # TldInfo, PeriodPriceMap
│   └── transfer.rs      # TransferCheckResult, …
└── ops/                 # impl DnaClient blocks, one file per domain
    ├── account.rs
    ├── availability.rs
    ├── contact.rs
    ├── domain.rs
    ├── nameserver.rs
    ├── tld.rs
    ├── transfer.rs
    └── util.rs          # Private parse helpers
```

---

## Running Tests

```bash
# Run all tests
cargo test

# Run a specific test file
cargo test --test test_domain

# Run inline unit tests only
cargo test --lib

# Run with output visible
cargo test -- --nocapture
```

---

## License

MIT — see [LICENSE](LICENSE).