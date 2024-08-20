use anyhow::Result;
use cloudflare::framework::{async_api::Client, auth, Environment};
use essentials::info;
use pem::{EncodeConfig, Pem};

use crate::{Certificate, CertificateResult};

mod api;

pub struct Cloudflare {
    pub client: Client,
}

impl Cloudflare {
    pub fn new(key: String) -> Result<Self> {
        let client = Client::new(
            auth::Credentials::Service { key },
            Default::default(),
            Environment::Production,
        )?;
        Ok(Self { client })
    }

    pub async fn create_certificate(
        &self,
        domain: String,
        csr: Option<&[u8]>,
    ) -> Result<CertificateResult> {
        let hostnames = if domain.chars().filter(|c| *c == '.').count() == 1 {
            vec![format!("*.{}", domain), domain]
        } else {
            vec![domain]
        };
        if let Some(csr) = csr {
            let cert = self.sign(hostnames, csr).await?;
            Ok(CertificateResult::Refreshed(cert))
        } else {
            let cert = Certificate::generate(hostnames.clone(), move |csr| async move {
                self.sign(hostnames, csr.der()).await
            })
            .await?;
            Ok(CertificateResult::New(cert))
        }
    }

    async fn sign(&self, hostnames: Vec<String>, csr_der: &[u8]) -> Result<String> {
        let p = Pem::new("CERTIFICATE REQUEST", csr_der);
        let csr = pem::encode_config(&p, EncodeConfig::new().set_line_ending(pem::LineEnding::LF));
        let endpoint = api::certificates::CreateCertificate {
            csr,
            hostnames,
            request_type: api::certificates::RequestType::OriginRsa,
            requested_validity: None,
        };
        info!("Signing certificate {:#?}", endpoint);
        let response = self.client.request(&endpoint).await?;
        Ok(response.result.certificate)
    }
}
