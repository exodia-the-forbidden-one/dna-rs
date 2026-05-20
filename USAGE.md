# Usage Guide

This document walks through every public method of `dna-rs` with runnable examples.

All examples assume the following setup at the top of the file:

```rust
use dna_rs::{DnaClient, DnaError};

#[tokio::main]
async fn main() -> Result<(), DnaError> {
    let client = DnaClient::new("YOUR-RESELLER-UUID", "YOUR-API-TOKEN")?;
    // ... examples below
    Ok(())
}
```

---

## Table of Contents

1. [Creating a Client](#1-creating-a-client)
2. [Account & Balance](#2-account--balance)
3. [Domain Availability](#3-domain-availability)
4. [Domain List](#4-domain-list)
5. [Domain Details](#5-domain-details)
6. [Domain Registration](#6-domain-registration)
7. [Domain Renewal](#7-domain-renewal)
8. [Contacts](#8-contacts)
9. [Name Servers](#9-name-servers)
10. [Glue Records (Child Name Servers)](#10-glue-records-child-name-servers)
11. [Transfer Management](#11-transfer-management)
12. [Lock & Privacy](#12-lock--privacy)
13. [TLD List & Pricing](#13-tld-list--pricing)
14. [Utilities](#14-utilities)
15. [Error Handling Reference](#15-error-handling-reference)

---

## 1. Creating a Client

```rust
use dna_rs::DnaClient;

// Production environment
let client = DnaClient::new("RESELLER-UUID", "API-TOKEN")?;

// OTE sandbox environment
let client = DnaClient::new_ote("RESELLER-UUID", "API-TOKEN")?;

// Custom base URL (proxy, mock server, etc.)
let client = DnaClient::with_url(
    "RESELLER-UUID",
    "API-TOKEN",
    "https://my-local-proxy/api/v1",
)?;
```

`DnaClient` is cheap to clone internally (it wraps a `reqwest::Client`). Create one instance and reuse it for the lifetime of your application.

---

## 2. Account & Balance

### Get full reseller profile

```rust
let details = client.get_reseller_details().await?;

println!("ID:   {}", details.id);
println!("Name: {}", details.name);

for balance in &details.balances {
    println!("{}: {} {}", balance.currency, balance.balance, balance.symbol);
}
// USD: 150.75 $
// TL:  4200.00 TL
```

**Return type:** `ResellerDetails`

| Field      | Type           | Description                                   |
| ---------- | -------------- | --------------------------------------------- |
| `id`       | `String`       | Reseller UUID                                 |
| `name`     | `String`       | Reseller display name                         |
| `active`   | `bool`         | Always `true` (API does not expose this flag) |
| `balances` | `Vec<Balance>` | All currency balances                         |

### Get balance for a specific currency

```rust
let bal = client.get_current_balance("USD").await?;

println!("{} {} (id: {})", bal.balance, bal.currency_name, bal.currency_id);
// 150.75 USD (id: 2)
```

Supported currency codes: `"USD"`, `"TRY"`, `"EUR"`, `"GBP"`.

**Return type:** `CurrentBalance`

| Field             | Type     |
| ----------------- | -------- |
| `balance`         | `f64`    |
| `currency_id`     | `u32`    |
| `currency_name`   | `String` |
| `currency_symbol` | `String` |

---

## 3. Domain Availability

```rust
let results = client
    .check_availability(
        &["example", "mycompany"],   // SLD labels (without TLD)
        &["com", "net", "io"],       // TLD list (with or without leading dot)
        1,                           // registration period (years) – fallback if API omits it
        "create",                    // command hint forwarded verbatim
    )
    .await?;

for r in &results {
    let price = r.price.map(|p| format!("${:.2}", p)).unwrap_or_else(|| "N/A".into());
    println!(
        "{}.{:<6} {:>12}  {}",
        r.domain_name, r.tld, r.status, price
    );
}
// example.com      available  $9.99
// example.net   notavailable  N/A
// example.io       available  $49.00
```

**Return type:** `Vec<AvailabilityResult>`

| Field         | Type             | Description                        |
| ------------- | ---------------- | ---------------------------------- |
| `tld`         | `String`         | Extension without leading dot      |
| `domain_name` | `String`         | SLD label                          |
| `status`      | `String`         | `"available"` or `"notavailable"`  |
| `command`     | `String`         | Echoed from the caller             |
| `period`      | `u32`            | Period from API or caller fallback |
| `is_fee`      | `bool`           | Premium domain flag                |
| `price`       | `Option<f64>`    | Registration price                 |
| `currency`    | `Option<String>` | Currency code                      |
| `reason`      | `Option<String>` | Reason if unavailable              |

---

## 4. Domain List

```rust
// Default pagination: MaxResultCount=200, SkipCount=0
let list = client.get_list(None).await?;

println!("Total domains: {}", list.total_count);

for domain in &list.domains {
    println!(
        "{:<30} expires: {}  locked: {}",
        domain.domain_name,
        domain.expiration_date,
        domain.lock_status,
    );
}
```

Override pagination:

```rust
let list = client
    .get_list(Some(&[("MaxResultCount", "50"), ("SkipCount", "100")]))
    .await?;
```

**Return type:** `DomainList`

| Field         | Type                 |
| ------------- | -------------------- |
| `domains`     | `Vec<DomainSummary>` |
| `total_count` | `u64`                |

Each `DomainSummary` has: `id`, `status`, `domain_name`, `auth_code`, `lock_status`, `privacy_protection_status`, `is_child_name_server`, `name_servers`, `start_date`, `expiration_date`, `remaining_days`.

---

## 5. Domain Details

```rust
let info = client.get_details("example.com").await?;

println!("Domain:  {}", info.domain_name);
println!("Status:  {}", info.status);
println!("Locked:  {}", info.lock_status);
println!("Expires: {}", info.expiration_date);
println!("NS:      {:?}", info.name_servers);
println!("Registrant handle: {}", info.contacts.registrant);
```

**Return type:** `DomainInfo`

| Field                       | Type                |
| --------------------------- | ------------------- |
| `id`                        | `u64`               |
| `status`                    | `String`            |
| `domain_name`               | `String`            |
| `auth_code`                 | `String`            |
| `lock_status`               | `bool`              |
| `privacy_protection_status` | `bool`              |
| `is_child_name_server`      | `bool`              |
| `name_servers`              | `Vec<String>`       |
| `contacts`                  | `ContactIds`        |
| `start_date`                | `String`            |
| `expiration_date`           | `String`            |
| `remaining_days`            | `i64`               |
| `additional`                | `serde_json::Value` |
| `child_name_servers`        | `Vec<ChildNs>`      |

`ContactIds` fields: `registrant`, `administrative`, `technical`, `billing` — all `String` (handle IDs).

### Sync from registry

Fetches fresh data from the upstream registry. Delegates to `get_details` internally.

```rust
let info = client.sync_from_registry("example.com").await?;
```

---

## 6. Domain Registration

Build contacts first, then register:

```rust
use dna_rs::models::contact::ContactInput;
use std::collections::HashMap;

let registrant = ContactInput {
    first_name:         "Ali".into(),
    last_name:          "Yılmaz".into(),
    company:            "Acme Ltd".into(),
    email:              "ali@acme.com".into(),
    address_line1:      "Atatürk Cad. No:1".into(),
    address_line2:      "".into(),
    city:               "Istanbul".into(),
    state:              "Istanbul".into(),
    country:            "TR".into(),
    zip_code:           "34000".into(),
    phone:              "5321234567".into(),
    phone_country_code: "90".into(),
    fax:                "".into(),
    fax_country_code:   "".into(),
};

let mut contacts = HashMap::new();
contacts.insert("Registrant",    registrant.clone());
contacts.insert("Administrative", registrant.clone());
contacts.insert("Technical",     registrant.clone());
contacts.insert("Billing",       registrant.clone());

let info = client
    .register_with_contact_info(
        "example.com",
        1,                // period in years
        contacts,
        None,             // name servers – None uses the API defaults
        true,             // EPP lock enabled
        false,            // privacy protection disabled
        None,             // additional attributes
    )
    .await?;

println!("Registered: {} (id: {})", info.domain_name, info.id);
```

Supply custom name servers:

```rust
let ns = Some(vec![
    "ns1.cloudflare.com".into(),
    "ns2.cloudflare.com".into(),
]);

let info = client
    .register_with_contact_info("example.com", 1, contacts, ns, true, false, None)
    .await?;
```

Supply additional TLD-specific attributes:

```rust
use serde_json::json;

let extra = Some(json!({ "trademarkName": "ACME", "trademarkDate": "2020-01-01" }));

let info = client
    .register_with_contact_info("example.com", 1, contacts, None, true, false, extra)
    .await?;
```

---

## 7. Domain Renewal

```rust
let result = client.renew("example.com", 1).await?;
println!("New expiry: {}", result.expiration_date);
```

**Return type:** `RenewResult { expiration_date: String }`

---

## 8. Contacts

### Fetch contacts

```rust
let contacts = client.get_contacts("example.com").await?;

// Keys: "Registrant", "Administrative", "Technical", "Billing"
if let Some(reg) = contacts.get("Registrant") {
    println!("{} {} <{}>", reg.first_name, reg.last_name, reg.email);
    println!("City: {}, Country: {}", reg.address.city, reg.address.country);
    println!("Phone: +{} {}", reg.phone.country_code, reg.phone.number);
}
```

### Update contacts

```rust
use dna_rs::models::contact::ContactInput;
use std::collections::HashMap;

let updated = ContactInput {
    first_name: "Mehmet".into(),
    last_name:  "Kaya".into(),
    email:      "mehmet@example.com".into(),
    city:       "Ankara".into(),
    country:    "TR".into(),
    // ... other fields
    ..ContactInput::default()
};

let mut contacts = HashMap::new();
contacts.insert("Registrant", updated);

let result = client.save_contacts("example.com", contacts).await?;
println!("Updated registrant: {}", result["Registrant"].first_name);
```

### Validate and normalise a contact

Before sending a contact to the API, you can normalise it — filling in Turkish defaults for any blank mandatory fields and splitting the phone number into country code and local number:

```rust
let raw = ContactInput {
    first_name: "".into(),          // will be filled with "Isimyok"
    phone:      "+90 532 123 45 67".into(), // will be split → cc="90", number="5321234567"
    ..ContactInput::default()
};

let clean = client.validate_contact(raw);
println!("Name:  {}", clean.first_name);         // Isimyok
println!("CC:    {}", clean.phone_country_code);  // 90
println!("Phone: {}", clean.phone);               // 5321234567
```

**Defaults applied when a field is blank:**

| Field                | Default                  |
| -------------------- | ------------------------ |
| `first_name`         | `"Isimyok"`              |
| `last_name`          | Copied from `first_name` |
| `address_line1`      | `"Addres yok"`           |
| `city`               | `"ISTANBUL"`             |
| `country`            | `"TR"`                   |
| `zip_code`           | `"34000"`                |
| `phone`              | `"5555555555"`           |
| `phone_country_code` | `"90"`                   |

**Phone normalisation rules:**

| Digits                       | Rule                                                       |
| ---------------------------- | ---------------------------------------------------------- |
| 10 digits                    | Country code cleared, local number kept as-is              |
| 11 digits starting with `9`  | First 2 digits → country code                              |
| 12 digits starting with `90` | First 2 digits → country code                              |
| Anything else                | Country code → `"90"`, number → stripped digits or default |

---

## 9. Name Servers

### Replace all name servers

```rust
let ns = client
    .modify_name_server(
        "example.com",
        vec!["ns1.cloudflare.com".into(), "ns2.cloudflare.com".into()],
    )
    .await?;

println!("Active NS: {:?}", ns);
```

Returns `Vec<String>` — the list confirmed by the API, or the supplied list if the API returns none.

---

## 10. Glue Records (Child Name Servers)

Use these when the name servers live inside the domain being delegated — e.g. `ns1.example.com` for `example.com`.

### Add a glue record

```rust
// IPv4
client
    .add_child_name_server("example.com", "ns1.example.com", "1.2.3.4")
    .await?;

// IPv6 — ip_version is detected automatically
client
    .add_child_name_server("example.com", "ns1.example.com", "2001:db8::1")
    .await?;
```

### Update a glue record IP

```rust
client
    .modify_child_name_server("example.com", "ns1.example.com", "5.6.7.8")
    .await?;
```

### Delete a glue record

```rust
client
    .delete_child_name_server("example.com", "ns1.example.com")
    .await?;
```

---

## 11. Transfer Management

### Check transferability

```rust
let check = client.check_transfer("example.com", "EPP-CODE").await?;

if check.transfer_available {
    println!("Domain is transferable");
} else {
    println!("Not transferable: {} ({})", check.message, check.message_key);
    if check.transfer_lock {
        println!("Reason: registrar lock is active");
    }
    if !check.auth_code_is_valid {
        println!("Reason: auth code is invalid");
    }
}
```

**Return type:** `TransferCheckResult`

| Field                    | Type     |
| ------------------------ | -------- |
| `transfer_available`     | `bool`   |
| `auth_code_is_required`  | `bool`   |
| `auth_code_is_valid`     | `bool`   |
| `user_transfer_required` | `bool`   |
| `transfer_lock`          | `bool`   |
| `message`                | `String` |
| `message_key`            | `String` |

### Initiate a transfer

```rust
use dna_rs::models::contact::ContactInput;
use std::collections::HashMap;

let mut contacts = HashMap::new();
contacts.insert("Registrant", ContactInput {
    first_name: "Ali".into(),
    // ...
    ..ContactInput::default()
});

let info = client
    .transfer("example.com", "EPP-CODE", 1, Some(contacts))
    .await?;

println!("Transfer initiated for: {}", info.domain_name);
```

Contacts are optional — pass `None` if the registry does not require them:

```rust
let info = client.transfer("example.com", "EPP-CODE", 1, None).await?;
```

### Cancel a pending incoming transfer

```rust
let status = client.cancel_transfer("example.com").await?;
println!("Status: {}", status); // "Cancelled"
```

### Approve an outgoing transfer

```rust
let status = client.approve_transfer("example.com").await?;
println!("Status: {}", status); // "Approved"
```

### Reject an outgoing transfer

```rust
let status = client.reject_transfer("example.com").await?;
println!("Status: {}", status); // "Rejected"
```

---

## 12. Lock & Privacy

### Registrar (theft protection) lock

```rust
// Enable
client.enable_theft_protection_lock("example.com").await?;

// Disable (required before initiating a transfer)
client.disable_theft_protection_lock("example.com").await?;
```

### WHOIS privacy protection

```rust
// Enable privacy
client
    .modify_privacy_protection_status("example.com", true, Some("Owner request"))
    .await?;

// Disable privacy
let status = client
    .modify_privacy_protection_status("example.com", false, None)
    .await?;

assert!(!status);
```

The `reason` parameter is accepted for API-compatibility but is not forwarded (the REST gateway does not expose this field).

---

## 13. TLD List & Pricing

```rust
let tlds = client.get_tld_list(100).await?;

for tld in &tlds {
    println!(".{} — status: {}", tld.tld, tld.status);

    if let Some(reg) = tld.pricing.get("registration") {
        for (period, price) in reg {
            println!("  {}yr: ${:.2}", period, price);
        }
    }
}
```

**Return type:** `Vec<TldInfo>`

| Field        | Type                              | Description                         |
| ------------ | --------------------------------- | ----------------------------------- |
| `id`         | `u32`                             | Sequential index starting at 1      |
| `tld`        | `String`                          | Extension without leading dot       |
| `status`     | `String`                          | e.g. `"Active"`                     |
| `min_char`   | `u32`                             | Minimum SLD length                  |
| `max_char`   | `u32`                             | Maximum SLD length                  |
| `min_period` | `u32`                             | Minimum registration period (years) |
| `max_period` | `u32`                             | Maximum registration period (years) |
| `pricing`    | `HashMap<String, PeriodPriceMap>` | Keyed by operation                  |
| `currencies` | `HashMap<String, String>`         | Currency per operation              |

**Pricing keys:** `"registration"`, `"renew"`, `"transfer"`, `"restore"`, `"refund"`, `"backorder"`.

`PeriodPriceMap` is `BTreeMap<u32, f64>` — sorted by period ascending.

---

## 14. Utilities

### Detect Turkish TLDs

```rust
assert!(client.is_tr_tld("example.com.tr"));
assert!(client.is_tr_tld("FIRMA.COM.TR"));  // case-insensitive
assert!(!client.is_tr_tld("example.com"));
```

---

## 15. Error Handling Reference

```rust
use dna_rs::DnaError;

async fn robust_example(client: &dna_rs::DnaClient) {
    match client.get_details("example.com").await {
        Ok(info) => {
            println!("Got details for {}", info.domain_name);
        }

        // The API returned a structured error (4xx / 5xx)
        Err(DnaError::Api { code, message, details }) => {
            eprintln!("[{code}] {message}");
            if !details.is_empty() {
                eprintln!("Details: {details}");
            }
            // Common codes:
            //   CREDENTIALS          – bad API key or reseller ID
            //   NOT_FOUND            – domain does not exist in the account
            //   INSUFFICIENT_BALANCE – not enough credit
            //   HTTP_500             – fallback when the gateway returns no JSON body
        }

        // Network failure: timeout, DNS error, TLS error
        Err(DnaError::Http(e)) => {
            eprintln!("Network error: {e}");
        }

        // The gateway returned 200 but the body was not valid JSON
        // or did not match the expected schema
        Err(DnaError::Deserialize(e)) => {
            eprintln!("Could not parse response: {e}");
        }

        // A required field was absent in an otherwise parseable response
        Err(DnaError::UnexpectedResponse(msg)) => {
            eprintln!("Unexpected response shape: {msg}");
        }

        // The caller passed an invalid argument (e.g. empty domain list)
        Err(DnaError::InvalidArgument(msg)) => {
            eprintln!("Bad argument: {msg}");
        }
    }
}
```

### Checking the error code without a full match

```rust
if let Err(DnaError::Api { ref code, .. }) = result {
    if code == "INSUFFICIENT_BALANCE" {
        // top up and retry
    }
}
```

### Propagating errors with `?`

All `DnaError` variants implement `std::error::Error`, so they compose naturally with `anyhow`, `eyre`, or any other error-handling crate:

```rust
// With anyhow
use anyhow::Result;

async fn my_task(client: &dna_rs::DnaClient) -> Result<()> {
    let info = client.get_details("example.com").await?;
    println!("{}", info.domain_name);
    Ok(())
}
```