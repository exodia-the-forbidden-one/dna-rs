//! `impl DnaClient` blocks, split by API domain.
//!
//! Each sub-module contains one focused `impl DnaClient` block and its
//! private helpers, keeping files short and easy to navigate.

pub(crate) mod account;
pub(crate) mod availability;
pub(crate) mod contact;
pub(crate) mod domain;
pub(crate) mod nameserver;
pub(crate) mod tld;
pub(crate) mod transfer;
pub(crate) mod util;
