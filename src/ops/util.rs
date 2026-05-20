//! Private helpers shared across `ops` sub-modules.
//!
//! Nothing in this file is part of the public API.

use std::collections::HashMap;
use std::net::IpAddr;

use serde_json::Value;

use crate::error::DnaResult;
use crate::models::contact::{
    ContactAddress, ContactInfo, ContactInput, ContactPayload, ContactPhone, ContactRaw,
    contact_type_to_api,
};
use crate::models::domain::{
    ChildNs, ChildNsRaw, ContactIds, DomainInfo, DomainInfoResponse, IpEntry,
};
use crate::models::tld::PeriodPriceMap;

// ── Domain parsing ────────────────────────────────────────────────────────────

pub(crate) fn parse_domain_info(raw: DomainInfoResponse) -> DnaResult<DomainInfo> {
    let contacts = parse_contact_ids(raw.contacts.as_deref().unwrap_or(&[]));
    let child_name_servers = raw
        .hosts
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(parse_child_ns)
        .collect();

    Ok(DomainInfo {
        id: raw.id.unwrap_or(0),
        status: raw
            .status
            .as_ref()
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        domain_name: raw.domain_name.or(raw.name).unwrap_or_default(),
        auth_code: raw.auth_code.or(raw.epp_code).unwrap_or_default(),
        lock_status: raw.lock_status.unwrap_or(false),
        privacy_protection_status: raw.privacy_protection_status.unwrap_or(false),
        is_child_name_server: raw.hosts.as_ref().map(|h| !h.is_empty()).unwrap_or(false),
        name_servers: raw.nameservers.unwrap_or_default(),
        contacts,
        start_date: raw.start_date.unwrap_or_default(),
        expiration_date: raw.expiration_date.unwrap_or_default(),
        remaining_days: raw.remaining_day.unwrap_or(0),
        additional: raw.additional_attributes.unwrap_or(Value::Null),
        child_name_servers,
    })
}

fn parse_child_ns(ns: &ChildNsRaw) -> ChildNs {
    let ips: Vec<String> = ns
        .ip_addresses
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|e| e.ip_address.clone())
        .collect();
    ChildNs {
        ns: ns.name.clone().unwrap_or_default(),
        ip: ips.last().cloned().unwrap_or_default(),
    }
}

pub(crate) fn parse_contact_ids(contacts: &[ContactRaw]) -> ContactIds {
    let mut ids = ContactIds::default();
    for c in contacts {
        let api_type = c.contact_type.as_deref().unwrap_or("");
        let id = c
            .handle
            .clone()
            .or_else(|| c.id.as_ref().and_then(Value::as_str).map(str::to_owned))
            .unwrap_or_default();
        match api_type {
            "Billing" => ids.billing = id,
            "Tech" => ids.technical = id,
            "Admin" => ids.administrative = id,
            "Registrant" => ids.registrant = id,
            _ => {}
        }
    }
    ids
}

// ── Contact parsing ───────────────────────────────────────────────────────────

pub(crate) fn parse_contacts(raw: Vec<ContactRaw>) -> HashMap<String, ContactInfo> {
    const TYPE_MAP: &[(&str, &str)] = &[
        ("Admin", "Administrative"),
        ("Billing", "Billing"),
        ("Tech", "Technical"),
        ("Registrant", "Registrant"),
    ];

    raw.into_iter()
        .map(|c| {
            let api_type = c.contact_type.clone().unwrap_or_default();
            let label = TYPE_MAP
                .iter()
                .find(|(k, _)| *k == api_type)
                .map(|(_, v)| *v)
                .unwrap_or(api_type.as_str())
                .to_owned();
            let info = parse_contact_info(&c, &label);
            (label, info)
        })
        .collect()
}

pub(crate) fn parse_contact_info(c: &ContactRaw, contact_type: &str) -> ContactInfo {
    ContactInfo {
        id: c
            .handle
            .clone()
            .or_else(|| c.id.as_ref().and_then(Value::as_str).map(str::to_owned))
            .unwrap_or_default(),
        status: c.status.clone().unwrap_or_else(|| "Active".into()),
        first_name: c.first_name.clone().unwrap_or_default(),
        last_name: c.last_name.clone().unwrap_or_default(),
        company: c
            .company_name
            .clone()
            .or_else(|| c.organization_name.clone())
            .unwrap_or_default(),
        email: c
            .e_mail
            .clone()
            .or_else(|| c.email_address.clone())
            .or_else(|| c.email.clone())
            .unwrap_or_default(),
        contact_type: contact_type.to_owned(),
        address: ContactAddress {
            line1: c
                .address
                .clone()
                .or_else(|| c.address_line1.clone())
                .unwrap_or_default(),
            line2: c.address_line2.clone().unwrap_or_default(),
            line3: c.address_line3.clone().unwrap_or_default(),
            city: c.city.clone().unwrap_or_default(),
            state: c
                .state
                .clone()
                .or_else(|| c.state_or_province.clone())
                .unwrap_or_default(),
            country: c
                .country
                .clone()
                .or_else(|| c.country_code.clone())
                .unwrap_or_default(),
            zip_code: c
                .postal_code
                .clone()
                .or_else(|| c.zip_code.clone())
                .unwrap_or_default(),
        },
        phone: ContactPhone {
            number: c
                .phone
                .clone()
                .or_else(|| c.phone_number.clone())
                .unwrap_or_default(),
            country_code: c.phone_country_code.clone().unwrap_or_default(),
            fax: c
                .fax
                .clone()
                .or_else(|| c.fax_number.clone())
                .unwrap_or_default(),
            fax_cc: c.fax_country_code.clone().unwrap_or_default(),
        },
        additional: c.additional_attributes.clone().unwrap_or(Value::Null),
    }
}

pub(crate) fn build_contact_payload(input: &ContactInput, type_name: &str) -> ContactPayload {
    let mut address = input.address_line1.clone();
    if !input.address_line2.trim().is_empty() {
        address.push(' ');
        address.push_str(&input.address_line2);
    }

    ContactPayload {
        contact_type: contact_type_to_api(type_name).to_owned(),
        first_name: input.first_name.clone(),
        last_name: input.last_name.clone(),
        company_name: input.company.clone(),
        e_mail: input.email.clone(),
        address,
        city: input.city.clone(),
        state: input.state.clone(),
        country: input.country.clone(),
        postal_code: input.zip_code.clone(),
        phone_country_code: input.phone_country_code.clone(),
        phone: input.phone.clone(),
        fax_country_code: input.fax_country_code.clone(),
        fax: input.fax.clone(),
    }
}

// ── TLD pricing parsing ───────────────────────────────────────────────────────

pub(crate) fn parse_tld_pricing(
    prices: &Option<Vec<Value>>,
) -> (HashMap<String, PeriodPriceMap>, HashMap<String, String>) {
    let mut pricing: HashMap<String, PeriodPriceMap> = HashMap::new();
    let mut currencies: HashMap<String, String> = HashMap::new();

    const PRICE_TYPES: &[(&str, &str)] = &[
        ("register", "registration"),
        ("renew", "renew"),
        ("transfer", "transfer"),
        ("restore", "restore"),
        ("refund", "refund"),
        ("backorder", "backorder"),
    ];

    let Some(prices_vec) = prices else {
        return (pricing, currencies);
    };
    let Some(first) = prices_vec.first() else {
        return (pricing, currencies);
    };

    for (api_type, out_type) in PRICE_TYPES {
        let Some(api_val) = first.get(api_type) else {
            continue;
        };
        if let Some(arr) = api_val.as_array() {
            for entry in arr {
                insert_price(entry, out_type, &mut pricing, &mut currencies);
            }
        } else if api_val.is_object() {
            insert_price(api_val, out_type, &mut pricing, &mut currencies);
        }
    }

    (pricing, currencies)
}

fn insert_price(
    v: &Value,
    out_type: &str,
    pricing: &mut HashMap<String, PeriodPriceMap>,
    currencies: &mut HashMap<String, String>,
) {
    let period = v
        .get("period")
        .and_then(Value::as_u64)
        .map(|p| p as u32)
        .unwrap_or(1)
        .max(1);
    let price = v.get("price").and_then(Value::as_f64).unwrap_or(0.0);
    let curr = v
        .get("currency")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();

    pricing
        .entry(out_type.to_owned())
        .or_default()
        .insert(period, price);
    if !curr.is_empty() {
        currencies.insert(out_type.to_owned(), curr);
    }
}

// ── Network helpers ───────────────────────────────────────────────────────────

/// Construct an [`IpEntry`] with the correct version tag.
pub(crate) fn ip_entry(ip: &str) -> IpEntry {
    let version = match ip.parse::<IpAddr>() {
        Ok(IpAddr::V4(_)) => "v4",
        Ok(IpAddr::V6(_)) => "v6",
        Err(_) => "v4",
    };
    IpEntry {
        ip_address: ip.to_owned(),
        ip_version: version.to_owned(),
    }
}

// ── Currency metadata ─────────────────────────────────────────────────────────

/// Map an ISO currency code to `(id, display_name, symbol)`.
pub(crate) fn currency_meta(code: &str) -> (u32, &'static str, &'static str) {
    match code {
        "USD" => (2, "USD", "$"),
        "TRY" => (1, "TL", "TL"),
        "EUR" => (3, "EUR", "€"),
        "GBP" => (4, "GBP", "£"),
        _ => (0, "USD", "$"),
    }
}
