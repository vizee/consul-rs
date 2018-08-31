use futures::prelude::*;
use hyper::client;
use hyper::header;
use hyper::{self, Body, Chunk, Method, Request, StatusCode, Uri};
use percent_encoding::{utf8_percent_encode, QUERY_ENCODE_SET};
use serde::de::DeserializeOwned;
use serde_json;
use std::marker::PhantomData;
use std::str::FromStr;
use std::time::Duration;

pub trait ExecFuture: Future<Item = (StatusCode, Chunk), Error = hyper::Error> {}

impl<T: Future<Item = (StatusCode, Chunk), Error = hyper::Error>> ExecFuture for T {}

#[derive(Debug)]
pub enum QueryError {
    Hyper(hyper::Error),
    Response(StatusCode, QueryMeta, Chunk),
    Json(serde_json::Error),
}

#[derive(Debug)]
pub struct QueryOption {
    pub wait_index: u64,
    pub wait_time: Option<Duration>,
    pub dc: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug)]
pub struct QueryMeta {
    pub last_index: u64,
}

impl Default for QueryMeta {
    fn default() -> Self {
        Self { last_index: 0 }
    }
}

fn parse_query_meta<T>(resp: &hyper::Response<T>) -> Result<QueryMeta, ()> {
    let mut last_index: u64 = 0;
    if let Some(v) = resp.headers().get("X-Consul-Index") {
        let s = v.to_str().map_err(|_| ())?;
        last_index = FromStr::from_str(s).map_err(|_| ())?;
    }
    Ok(QueryMeta { last_index })
}

pub struct QueryFuture<T> {
    inner: Box<Future<Item = (StatusCode, QueryMeta, Chunk), Error = QueryError> + Send>,
    dummy: PhantomData<T>,
}

impl<T> Future for QueryFuture<T>
where
    T: DeserializeOwned,
{
    type Item = (QueryMeta, T);
    type Error = QueryError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let (status, meta, chunk) = try_ready!(self.inner.poll());
        if !status.is_success() {
            Err(QueryError::Response(status, meta, chunk))
        } else {
            match serde_json::from_slice(chunk.as_ref()) {
                Ok(v) => Ok(Async::Ready((meta, v))),
                Err(e) => Err(QueryError::Json(e)),
            }
        }
    }
}

pub struct ClientRequest<'a> {
    method: Method,
    path: &'a str,
    params: Vec<&'a str>,
    body: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct Client {
    base: String,
    token: Option<String>,
    hc: hyper::Client<client::HttpConnector>,
}

impl Client {
    pub fn new(base: String, token: Option<String>) -> Client {
        let mut base = base;
        if !base.ends_with("/") {
            base += "/";
        }
        Client {
            base,
            token,
            hc: hyper::Client::builder().keep_alive(true).build_http(),
        }
    }

    fn send(&self, req: ClientRequest) -> client::ResponseFuture {
        let params = &req.params;
        let n: usize = params.iter().map(|s| s.len()).sum();
        let mut u = String::with_capacity(self.base.len() + req.path.len() + params.len() * 2 + n);
        u.push_str(self.base.as_str());
        u.push_str(req.path);
        u.push('?');
        let mut i = 0;
        while i < params.len() {
            if i > 0 {
                u.push('&');
            }
            u.push_str(params[i]);
            u.push('=');
            u.push_str(
                utf8_percent_encode(params[i + 1], QUERY_ENCODE_SET)
                    .to_string()
                    .as_str(),
            );
            i += 2;
        }
        let body = match req.body {
            Some(body) => Body::from(body),
            None => Body::empty(),
        };
        let mut hr = Request::new(body);
        *hr.uri_mut() = Uri::from_str(u.as_str()).unwrap();
        *hr.method_mut() = req.method;
        hr.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        if let Some(ref t) = self.token {
            hr.headers_mut().insert(
                header::HeaderName::from_static("X-Consul-Token"),
                header::HeaderValue::from_str(t.as_str()).unwrap(),
            );
        }
        self.hc.request(hr)
    }

    pub(super) fn exec(
        &self,
        method: Method,
        path: &str,
        params: Vec<&str>,
        body: Option<Vec<u8>>,
    ) -> impl ExecFuture {
        self.send(ClientRequest {
            method,
            path,
            params,
            body,
        }).and_then(|resp| {
            let status = resp.status();
            resp.into_body().concat2().map(move |chunk| (status, chunk))
        })
    }

    pub(super) fn query<T: DeserializeOwned>(
        &self,
        path: &str,
        params: Vec<&str>,
        qo: Option<QueryOption>,
    ) -> QueryFuture<T> {
        let (index, wait);
        let mut params = params;
        if let Some(ref qo) = qo {
            if let Some(ref v) = qo.dc {
                params.push("dc");
                params.push(v.as_str());
            }
            if let Some(ref v) = qo.tag {
                params.push("tag");
                params.push(v.as_str());
            }
            if qo.wait_index != 0 {
                index = qo.wait_index.to_string();
                params.push("index");
                params.push(index.as_str());
            }
            if let Some(ref v) = qo.wait_time {
                wait = v.as_secs().to_string() + "s";
                params.push("wait");
                params.push(wait.as_str());
            }
        }
        QueryFuture {
            inner: Box::new(
                self.send(ClientRequest {
                    method: Method::GET,
                    path,
                    params,
                    body: None,
                }).and_then(|resp| {
                    let status = resp.status();
                    let meta = parse_query_meta(&resp).unwrap_or_default();
                    resp.into_body()
                        .concat2()
                        .map(move |chunk| (status, meta, chunk))
                }).map_err(|e| QueryError::Hyper(e)),
            ),
            dummy: PhantomData,
        }
    }
}
