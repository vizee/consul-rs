use client::*;
use futures::prelude::*;
use hyper::{self, Method};
use serde_json;
use types::*;

impl Client {
    pub fn agent_check_pass(&self, id: &str, note: &str) -> impl ExecFuture {
        self.exec(
            Method::PUT,
            format!("/v1/agent/check/pass/{}", id).as_str(),
            vec!["note", note],
            None,
        )
    }

    pub fn agent_service_register(&self, service: &AgentService) -> impl ExecFuture {
        self.exec(
            Method::PUT,
            "v1/agent/service/register",
            vec![],
            Some(serde_json::to_vec(service).unwrap()),
        )
    }

    pub fn agent_service_deregister(&self, id: &str) -> impl ExecFuture {
        self.exec(
            Method::PUT,
            format!("/v1/agent/service/deregister/{}", id).as_str(),
            vec![],
            None,
        )
    }

    pub fn catalog_service(
        &self,
        service: &str,
        tag: &str,
        qo: Option<QueryOption>,
    ) -> QueryFuture<Vec<CatalogService>> {
        self.query(
            format!("/v1/catalog/service/{}", service).as_str(),
            vec!["tag", tag],
            qo,
        )
    }

    pub fn kv_list(&self, prefix: &str, qo: Option<QueryOption>) -> QueryFuture<Vec<KVPair>> {
        self.query(
            format!("/v1/kv/{}", prefix.trim_left_matches('/')).as_str(),
            vec!["recurse", ""],
            qo,
        )
    }

    pub fn kv_keys(&self, prefix: &str, qo: Option<QueryOption>) -> QueryFuture<Vec<String>> {
        self.query(
            format!("/v1/kv/{}", prefix.trim_left_matches('/')).as_str(),
            vec!["keys", ""],
            qo,
        )
    }

    pub fn kv_get(&self, prefix: &str, qo: Option<QueryOption>) -> QueryFuture<Vec<KVPair>> {
        self.query(
            format!("/v1/kv/{}", prefix.trim_left_matches('/')).as_str(),
            vec![],
            qo,
        )
    }

    fn kv_put_inner(
        &self,
        key: &str,
        value: &[u8],
        params: Vec<&str>,
    ) -> impl Future<Item = bool, Error = hyper::Error> {
        self.exec(
            Method::PUT,
            format!("/v1/kv/{}", key.trim_left_matches('/')).as_str(),
            params,
            Some(value.to_vec()),
        ).map(|(status, chunk)| status.is_success() && chunk.as_ref() == b"true")
    }

    pub fn kv_put(
        &self,
        key: &str,
        value: &[u8],
    ) -> impl Future<Item = bool, Error = hyper::Error> {
        self.kv_put_inner(key, value, vec![])
    }

    pub fn kv_cas(
        &self,
        key: &str,
        value: &[u8],
        idx: u64,
    ) -> impl Future<Item = bool, Error = hyper::Error> {
        self.kv_put_inner(key, value, vec!["cas", idx.to_string().as_str()])
    }
}
