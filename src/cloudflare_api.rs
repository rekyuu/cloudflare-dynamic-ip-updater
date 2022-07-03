use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudflareResponse<T> {
    pub result: T,
    pub success: bool,
    pub errors: Vec<CloudflareError>,
    pub messages: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudflareError {
    pub code: i64,
    pub message: String
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudflareDnsRecord {
    #[serde(rename = "type")]
    pub dns_type: String,
    pub name: String,
    pub content: String,
    pub ttl: i64,
    pub proxied: bool
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudflareDnsResult {
    pub id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub name: String,
    #[serde(rename = "type")]
    pub dns_type: String,
    pub content: String,
    pub proxiable: bool,
    pub proxied: bool,
    pub ttl: i64,
    pub locked: bool,
    pub meta: Meta,
    pub created_on: String,
    pub modified_on: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub auto_added: bool,
    pub managed_by_apps: bool,
    pub managed_by_argo_tunnel: bool,
    pub source: String,
}