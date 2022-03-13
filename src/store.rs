use axum::body::{Bytes, HttpBody};
use dashmap::DashMap;
use hyper::{Body, HeaderMap, Method, Request, StatusCode};
use lazy_static::lazy_static;

// CacheKey, (method, path, body)
#[derive(PartialEq, Eq, Hash)]
pub struct CacheKey(Method, String, Option<Bytes>);

impl CacheKey {
    pub async fn read_from_req(req: &mut Request<Body>) -> Self {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let body = req.body_mut().data().await.map(|result| result.unwrap());
        Self(method, path, body)
    }

    pub fn get_body(&self) -> Bytes {
        match &self.2 {
            Some(bs) => bs.clone(),
            None => Bytes::new(),
        }
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
