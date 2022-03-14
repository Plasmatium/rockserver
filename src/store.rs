
use std::io::{ Read};

use axum::body::{Bytes, HttpBody};
use bytes::Buf;
use dashmap::DashMap;
use hyper::{Body, HeaderMap, Method, Request, StatusCode, body::to_bytes};
use lazy_static::lazy_static;
use tracing::info;

// CacheKey, (method, path, body)
#[derive(PartialEq, Eq, Hash)]
pub struct CacheKey(Method, String, Bytes);

impl CacheKey {
    pub async fn read_from_req(req: &mut Request<Body>) -> Self {
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        let body = to_bytes(req.body_mut()).await.expect("read body failed");
        Self(method, path, body)
    }

    pub fn get_body(&self) -> &Bytes {
        &self.2
    }
}

// CacheValue, (header, status_code, body)
#[derive(Clone)]
pub struct CacheValue(pub HeaderMap, pub StatusCode, pub Bytes, pub CacheConfig);

#[derive(Default, Clone)]
pub struct CacheConfig {
    pub match_headers: Vec<String>,
}

#[derive(Default)]
pub struct Store(pub DashMap<CacheKey, CacheValue>);

impl Store {}

lazy_static! {
    pub static ref GLOBAL_STORE: Store = Store(Default::default());
}
