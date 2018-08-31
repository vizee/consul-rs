use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CatalogService {
    #[serde(rename = "ServiceID")]
    pub service_id: String,
    pub service_name: String,
    pub service_address: String,
    pub service_port: i32,
    pub service_tags: Vec<String>,
    pub service_meta: Option<BTreeMap<String, String>>,
    pub create_index: u64,
    pub modify_index: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AgentServiceCheck {
    #[serde(rename = "TTL")]
    ttl: String,
    deregister_critical_service_after: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AgentService {
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: i32,
    pub tags: Vec<String>,
    pub meta: Option<BTreeMap<String, String>>,
    pub check: AgentServiceCheck,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KVPair {
    pub lock_index: u64,
    pub key: String,
    pub flags: u64,
    pub value: String,
    pub create_index: u64,
    pub modify_index: u64,
}
