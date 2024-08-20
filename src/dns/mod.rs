#[cfg(feature = "dns-cloudflare")]
pub mod cloudflare;
pub mod manual;

mod utils;

use anyhow::Result;
use essentials::info;
use std::future::Future;
use utils::{check_txt_value, RFindNth};

const DNS_RECORD_NAME: &str = "_acme-challenge";

pub enum Clients {
    #[cfg(feature = "dns-cloudflare")]
    Cloudflare(cloudflare::Client),
    Manual(manual::Client),
}

pub trait Client {
    fn wrap(self) -> Clients;

    fn create_record(
        &self,
        domain: String,
        name: String,
        value: String,
    ) -> impl Future<Output = Result<(String, String)>>;

    fn delete_record(&self, domain_id: String, dns_id: String) -> impl Future<Output = Result<()>>;
}

impl Clients {
    pub async fn create_record(&self, domain: &String, value: String) -> Result<(String, String)> {
        info!("Creating DNS record for domain={}", domain);
        let (domain_name, name) = match domain.rfind_nth('.', 1) {
            Some(i) => (
                domain[i + 1..].to_string(),
                format!("{}.{}", DNS_RECORD_NAME, &domain[..i]),
            ),
            None => (domain.clone(), DNS_RECORD_NAME.to_owned()),
        };
        let domain = format!("{}.{}", name, domain_name);
        let result = {
            let (domain, name, value) = (domain_name, name.clone(), value.clone());
            use Clients::*;
            match self {
                #[cfg(feature = "dns-cloudflare")]
                Cloudflare(client) => client.create_record(domain, name, value).await?,
                Manual(client) => client.create_record(domain, name, value).await?,
            }
        };
        info!("DNS record created: {:#?}", result);
        // wait for DNS record to propagate
        check_txt_value(&domain, &value, 10).await?;
        Ok(result)
    }

    pub async fn delete_record(&self, domain_id: String, dns_id: String) -> Result<()> {
        use Clients::*;
        match self {
            #[cfg(feature = "dns-cloudflare")]
            Cloudflare(client) => client.delete_record(domain_id, dns_id).await,
            Manual(client) => client.delete_record(domain_id, dns_id).await,
        }
    }

    pub fn manual() -> Self {
        manual::Client::new().wrap()
    }

    #[cfg(feature = "dns-cloudflare")]
    pub fn cloudflare(token: String) -> Result<Self> {
        let client = cloudflare::Client::new(token)?;
        Ok(client.wrap())
    }
}
