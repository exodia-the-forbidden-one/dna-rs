use std::collections::HashMap;

use crate::client::DnaClient;
use crate::error::DnaResult;
use crate::models::contact::{
    ContactInfo, ContactInput, ContactPayload, SaveContactsPayload, SaveContactsResponse,
};
use crate::models::domain::DomainInfoResponse;
use crate::ops::util::{build_contact_payload, parse_contacts};

impl DnaClient {
    /// Fetch all contacts for a domain.
    ///
    /// Returns a map of `"Administrative" | "Billing" | "Technical" | "Registrant"`
    /// to [`ContactInfo`].
    pub async fn get_contacts(&self, domain_name: &str) -> DnaResult<HashMap<String, ContactInfo>> {
        let query = [("DomainName", domain_name)];
        let raw: DomainInfoResponse = self.http.get("domains/info", Some(&query)).await?;
        Ok(parse_contacts(raw.contacts.unwrap_or_default()))
    }

    /// Update all contacts for a domain.
    ///
    /// `contacts` maps a type label (`"Registrant"`, `"Administrative"`, etc.)
    /// to the new contact data.
    pub async fn save_contacts(
        &self,
        domain_name: &str,
        contacts: HashMap<&str, ContactInput>,
    ) -> DnaResult<HashMap<String, ContactInfo>> {
        let payload_contacts: Vec<ContactPayload> = contacts
            .iter()
            .map(|(t, c)| build_contact_payload(c, t))
            .collect();

        let payload = SaveContactsPayload {
            domain_name: domain_name.into(),
            contacts: payload_contacts,
        };

        let resp: SaveContactsResponse = self.http.put("domains/contacts/update", &payload).await?;

        Ok(parse_contacts(resp.contacts.unwrap_or_default()))
    }
}
