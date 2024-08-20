use std::future::Future;

use anyhow::Result;
use rcgen::{CertificateParams, CertificateSigningRequest, DistinguishedName, KeyPair};

#[derive(Debug)]
pub struct Certificate {
    pub private_key: String,
    pub certificate: String,
    pub csr_der: Box<[u8]>,
}

#[derive(Debug)]
pub enum CertificateResult {
    Refreshed(String),
    New(Certificate),
}

impl Certificate {
    pub async fn generate<R, S>(subject_alt_names: impl Into<Vec<String>>, sign: S) -> Result<Self>
    where
        R: Future<Output = Result<String>>,
        S: FnOnce(CertificateSigningRequest) -> R,
    {
        let mut params = CertificateParams::new(subject_alt_names)?;
        params.distinguished_name = DistinguishedName::new();
        let private_key = KeyPair::generate()?;
        let csr = params.serialize_request(&private_key)?;
        let csr_der = csr.der().to_vec().into_boxed_slice();
        let cert_chain_pem = sign(csr).await?;
        Ok(Self {
            private_key: private_key.serialize_pem(),
            certificate: cert_chain_pem,
            csr_der,
        })
    }
}
