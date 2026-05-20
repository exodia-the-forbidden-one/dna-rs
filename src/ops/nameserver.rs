use serde_json::{Value, json};

use crate::client::DnaClient;
use crate::error::DnaResult;
use crate::models::domain::{ChildNsPayload, ModifyChildNsPayload, ModifyNsPayload, NsResponse};
use crate::ops::util::ip_entry;

impl DnaClient {
    /// Replace all name-servers for a domain.
    pub async fn modify_name_server(
        &self,
        domain_name: &str,
        name_servers: Vec<String>,
    ) -> DnaResult<Vec<String>> {
        let payload = ModifyNsPayload {
            domain_name: domain_name.into(),
            name_servers: name_servers.clone(),
        };
        let resp: NsResponse = self.http.put("domains/dns/name-server", &payload).await?;
        Ok(resp.name_servers.unwrap_or(name_servers))
    }

    /// Add a glue (child) name-server to a domain.
    pub async fn add_child_name_server(
        &self,
        domain_name: &str,
        host_name: &str,
        ip_address: &str,
    ) -> DnaResult<()> {
        let payload = ChildNsPayload {
            domain_name: domain_name.into(),
            host_name: host_name.into(),
            ip_addresses: vec![ip_entry(ip_address)],
        };
        let _: Value = self.http.post("domains/dns/host", &payload).await?;
        Ok(())
    }

    /// Delete a glue name-server from a domain.
    pub async fn delete_child_name_server(
        &self,
        domain_name: &str,
        host_name: &str,
    ) -> DnaResult<()> {
        let body = json!({ "domainName": domain_name, "hostName": host_name });
        let _: Value = self.http.delete("domains/dns/host", Some(&body)).await?;
        Ok(())
    }

    /// Update the IP address of a glue name-server.
    pub async fn modify_child_name_server(
        &self,
        domain_name: &str,
        host_name: &str,
        ip_address: &str,
    ) -> DnaResult<()> {
        let payload = ModifyChildNsPayload {
            domain_name: domain_name.into(),
            host_name: host_name.into(),
            new_host_name: host_name.into(),
            ip_addresses: vec![ip_entry(ip_address)],
        };
        let _: Value = self.http.put("domains/dns/host", &payload).await?;
        Ok(())
    }
}
