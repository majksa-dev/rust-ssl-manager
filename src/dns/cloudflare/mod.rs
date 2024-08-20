mod api;

use anyhow::{anyhow, Result};
use api::{dns, zone};
use cloudflare::framework::{async_api::Client as ApiClient, auth::Credentials, Environment};
use essentials::{debug, info};

pub struct Client {
    client: ApiClient,
}

impl Client {
    pub fn new(token: String) -> Result<Self> {
        let credentials = Credentials::UserAuthToken { token };
        let client = ApiClient::new(credentials, Default::default(), Environment::Production)?;

        Ok(Self { client })
    }

    pub async fn get_zone_id(&self, domain: String) -> Result<String> {
        let endpoint = zone::ListZones {
            params: zone::ListZonesParams {
                name: Some(domain.clone()),
                per_page: Some(1),
                ..Default::default()
            },
        };
        let response = self.client.request(&endpoint).await?;
        let zone = response
            .result
            .0
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("zone not found for domain={domain}"))?;
        info!("Zone found: {:#?}", zone);
        Ok(zone.id)
    }
}

impl super::Client for Client {
    fn wrap(self) -> super::Clients {
        super::Clients::Cloudflare(self)
    }

    async fn create_record(
        &self,
        domain: String,
        name: String,
        value: String,
    ) -> Result<(String, String)> {
        let zone_identifier = self.get_zone_id(domain).await?;
        let endpoint = dns::CreateDnsRecord {
            zone_identifier: &zone_identifier,
            params: dns::CreateDnsRecordParams {
                name: &name,
                content: dns::DnsContent::TXT { content: value },
                priority: None,
                proxied: None,
                ttl: None,
            },
        };
        debug!("Creating DNS record: {:#?}", endpoint);
        let dns = self.client.request(&endpoint).await?;
        Ok((zone_identifier, dns.result.id))
    }

    async fn delete_record(&self, domain_id: String, dns_id: String) -> Result<()> {
        let endpoint = cloudflare::endpoints::dns::DeleteDnsRecord {
            zone_identifier: &domain_id,
            identifier: &dns_id,
        };
        self.client.request(&endpoint).await?;
        Ok(())
    }
}
