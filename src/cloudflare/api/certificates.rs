use cloudflare::framework::{
    endpoint::{spec::EndpointSpec, Method},
    response::ApiResult,
};
use serde::{Deserialize, Serialize};

/// Create Origin certificate
/// <https://developers.cloudflare.com/api/operations/origin-ca-create-certificate>
#[derive(Serialize, Clone, Debug)]
pub struct CreateCertificate {
    pub csr: String,
    pub hostnames: Vec<String>,
    pub request_type: RequestType,
    pub requested_validity: Option<usize>,
}

impl EndpointSpec<Certificate> for CreateCertificate {
    fn method(&self) -> Method {
        Method::POST
    }
    fn path(&self) -> String {
        "certificates".to_owned()
    }
    #[inline]
    fn body(&self) -> Option<String> {
        let body = serde_json::to_string(&self).unwrap();
        Some(body)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum RequestType {
    #[serde(rename = "origin-rsa")]
    OriginRsa,
    #[serde(rename = "origin-ecc")]
    OriginRcc,
    #[serde(rename = "keyless-certificate")]
    KeylessCertificate,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Certificate {
    pub certificate: String,
    pub csr: String,
    pub expires_on: String,
    pub hostnames: Vec<String>,
    pub id: String,
    pub request_type: RequestType,
    pub requested_validity: usize,
}

impl ApiResult for Certificate {}
