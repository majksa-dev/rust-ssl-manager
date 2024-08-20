mod certificate;
mod challenges;
mod ready;

pub use certificate::Certificate;
pub use challenges::CertificateResult;

use anyhow::{bail, Result};
use challenges::CreateChallenges;
use essentials::{error, info, warn};
use instant_acme::{Account, Identifier, LetsEncrypt, NewAccount, NewOrder, OrderStatus};
use std::{
    fs::{self, File},
    path::Path,
};

use crate::dns::{self};

pub struct AcmeClient {
    account: Account,
}

impl AcmeClient {
    pub async fn new(email: &str, credentials_path: &str) -> Result<Self> {
        let path = Path::new(credentials_path);
        let account = if path.exists() {
            let file = fs::read(path)?;
            let credentials = serde_json::from_slice(&file)?;
            Account::from_credentials(credentials).await?
        } else {
            let (account, credentials) = Account::create(
                &NewAccount {
                    contact: &[&format!("mailto:{}", email)],
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                LetsEncrypt::Production.url(),
                None,
            )
            .await?;
            let file = File::create(path)?;
            serde_json::to_writer(file, &credentials)?;
            account
        };

        Ok(Self { account })
    }

    pub async fn request_certificate(
        &self,
        dns_client: &dns::Clients,
        domain: String,
        csr: Option<&[u8]>,
    ) -> Result<CertificateResult> {
        let identifier = Identifier::Dns(domain);
        let mut order = self
            .account
            .new_order(&NewOrder {
                identifiers: &[identifier],
            })
            .await?;

        let state = order.state();
        if !matches!(state.status, OrderStatus::Pending) {
            error!(?state, "unexpected order status");
            bail!("unexpected order status: {:?}", state.status);
        }

        let (names, dns_ids) = order.create_challenges(dns_client).await?;
        let certificate = order.finish_challenges(names, csr).await;
        info!(
            ?dns_ids,
            ?certificate,
            "tried to issue certificate, deleting DNS records",
        );
        for dns_id in dns_ids {
            if let Err(err) = dns_client.delete_record(dns_id.0, dns_id.1).await {
                warn!(?err, "failed to delete DNS record");
            }
        }
        certificate
    }
}
