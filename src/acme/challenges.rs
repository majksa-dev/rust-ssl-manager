use std::time::Duration;

use anyhow::{Context, Result};
use essentials::{info, warn};
use instant_acme::{AuthorizationStatus, ChallengeType, Identifier, Order};
use tokio::time::sleep;

use crate::{certificate::CertificateResult, dns::Clients, Certificate};

use super::ready::Ready;

pub(super) trait CreateChallenges {
    async fn create_challenges(
        &mut self,
        dns_client: &Clients,
    ) -> Result<(Vec<String>, Vec<(String, String)>)>;

    async fn finish_challenges(
        &mut self,
        names: Vec<String>,
        csr: Option<&[u8]>,
    ) -> Result<CertificateResult>;
}

impl CreateChallenges for Order {
    async fn create_challenges(
        &mut self,
        dns_client: &Clients,
    ) -> Result<(Vec<String>, Vec<(String, String)>)> {
        let authorizations = self.authorizations().await?;
        let mut challenges = Vec::with_capacity(authorizations.len());
        let mut dns_ids = Vec::with_capacity(authorizations.len());
        for authz in &authorizations {
            match authz.status {
                AuthorizationStatus::Pending => {}
                AuthorizationStatus::Valid => continue,
                _ => todo!(),
            }

            // We'll use the DNS challenges for this example, but you could
            // pick something else to use here.

            let challenge = match authz
                .challenges
                .iter()
                .find(|c| c.r#type == ChallengeType::Dns01)
            {
                Some(challenge) => challenge,
                None => {
                    warn!("no DNS challenge found for authorization");
                    continue;
                }
            };

            let Identifier::Dns(identifier) = &authz.identifier;

            let dns_id = match dns_client
                .create_record(identifier, self.key_authorization(challenge).dns_value())
                .await
            {
                Ok(dns_id) => dns_id,
                Err(err) => {
                    warn!(?err, "failed to create DNS record");
                    continue;
                }
            };
            dns_ids.push(dns_id);
            challenges.push((identifier, &challenge.url));
        }

        info!("challenges created: {:#?}", challenges);

        for (_, url) in &challenges {
            if let Err(err) = self.set_challenge_ready(url).await {
                warn!(?err, "failed to set challenge ready");
                for dns_id in dns_ids {
                    match dns_client.delete_record(dns_id.0, dns_id.1).await {
                        Ok(dns_id) => dns_id,
                        Err(err) => {
                            warn!(?err, "failed to create DNS record");
                            continue;
                        }
                    };
                }
                return Err(err).with_context(|| "failed to set challenge ready");
            }
        }
        Ok((
            challenges
                .into_iter()
                .map(|(name, _)| name.to_owned())
                .collect(),
            dns_ids,
        ))
    }

    async fn finish_challenges(
        &mut self,
        names: Vec<String>,
        csr: Option<&[u8]>,
    ) -> Result<CertificateResult> {
        self.ready().await?;
        if let Some(csr) = csr {
            let cert = sign(self, csr).await?;
            Ok(CertificateResult::Refreshed(cert))
        } else {
            let cert =
                Certificate::generate(names, move |csr| async move { sign(self, csr.der()).await })
                    .await?;
            Ok(CertificateResult::New(cert))
        }
    }
}

async fn sign(order: &mut Order, csr_der: &[u8]) -> Result<String> {
    order.finalize(csr_der).await?;
    let cert_chain_pem = loop {
        match order.certificate().await? {
            Some(cert_chain_pem) => break cert_chain_pem,
            None => sleep(Duration::from_secs(1)).await,
        }
    };
    Ok(cert_chain_pem)
}
