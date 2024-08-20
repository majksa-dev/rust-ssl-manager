use cloudflare::{
    endpoints::zone::{ListZonesOrder, Status},
    framework::{
        endpoint::{serialize_query, spec::EndpointSpec, Method},
        response::ApiResult,
        OrderDirection, SearchMatch,
    },
};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize)]
pub struct Zone {
    pub id: String,
}

#[derive(Debug)]
pub struct Zones(pub Vec<Zone>);

impl<'de> Deserialize<'de> for Zones {
    fn deserialize<D>(deserializer: D) -> Result<Zones, D::Error>
    where
        D: Deserializer<'de>,
    {
        let zones = Vec::<Zone>::deserialize(deserializer)?;
        Ok(Zones(zones))
    }
}

impl ApiResult for Zone {}
impl ApiResult for Zones {}

#[derive(Debug)]
pub struct ListZones {
    pub params: ListZonesParams,
}

impl EndpointSpec<Zones> for ListZones {
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> String {
        "zones".to_string()
    }
    #[inline]
    fn query(&self) -> Option<String> {
        serialize_query(&self.params)
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ListZonesParams {
    pub name: Option<String>,
    pub status: Option<Status>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order: Option<ListZonesOrder>,
    pub direction: Option<OrderDirection>,
    #[serde(rename = "match")]
    pub search_match: Option<SearchMatch>,
}
