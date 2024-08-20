use cloudflare::framework::{
    endpoint::{spec::EndpointSpec, Method},
    response::ApiResult,
};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};

/// Create DNS Record
/// <https://api.cloudflare.com/#dns-records-for-a-zone-create-dns-record>
#[derive(Debug)]
pub struct CreateDnsRecord<'a> {
    pub zone_identifier: &'a str,
    pub params: CreateDnsRecordParams<'a>,
}

impl<'a> EndpointSpec<DnsRecord> for CreateDnsRecord<'a> {
    fn method(&self) -> Method {
        Method::POST
    }
    fn path(&self) -> String {
        format!("zones/{}/dns_records", self.zone_identifier)
    }
    #[inline]
    fn body(&self) -> Option<String> {
        let body = serde_json::to_string(&self.params).unwrap();
        Some(body)
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct CreateDnsRecordParams<'a> {
    /// Time to live for DNS record. Value of 1 is 'automatic'
    pub ttl: Option<u32>,
    /// Used with some records like MX and SRV to determine priority.
    /// If you do not supply a priority for an MX record, a default value of 0 will be set
    pub priority: Option<u16>,
    /// Whether the record is receiving the performance and security benefits of Cloudflare
    pub proxied: Option<bool>,
    /// DNS record name
    pub name: &'a str,
    /// Type of the DNS record that also holds the record value
    #[serde(flatten)]
    pub content: DnsContent,
}

/// Type of the DNS record, along with the associated value.
/// When we add support for other types (LOC/SRV/...), the `meta` field should also probably be encoded
/// here as an associated, strongly typed value.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
#[allow(clippy::upper_case_acronyms)]
pub enum DnsContent {
    A { content: Ipv4Addr },
    AAAA { content: Ipv6Addr },
    CNAME { content: String },
    NS { content: String },
    MX { content: String, priority: u16 },
    TXT { content: String },
    SRV { content: String },
}

#[derive(Deserialize, Debug)]
pub struct DnsRecord {
    pub id: String,
}

impl ApiResult for DnsRecord {}
