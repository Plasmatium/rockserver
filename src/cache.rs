use axum::http::{uri::PathAndQuery, HeaderValue};
use bytes::Bytes;
use dashmap::DashMap;
use hyper::{body::to_bytes, Body, HeaderMap, Method, Request, StatusCode};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Serialize, Deserialize, Clone)]
pub enum TaggedBody {
    Base64(String),
    String(String),
}

impl From<&Bytes> for TaggedBody {
    fn from(bs: &Bytes) -> Self {
        let bs = bs.as_ref();
        let converted = String::from_utf8(bs.to_vec());
        match converted {
            Ok(data) => TaggedBody::String(data),
            _ => {
                debug!("body is not a valid utf8, converting to base64");
                TaggedBody::Base64(base64::encode(bs))
            }
        }
    }
}

impl From<&TaggedBody> for Bytes {
    fn from(tb: &TaggedBody) -> Self {
        match tb {
            TaggedBody::String(body) => Bytes::copy_from_slice(body.as_bytes()),
            TaggedBody::Base64(body) => {
                let decoded =
                    base64::decode(body).unwrap_or(b"failed to decode base64 body".to_vec());
                Bytes::from(decoded)
            }
        }
    }
}

impl AsRef<[u8]> for TaggedBody {
    fn as_ref(&self) -> &[u8] {
        let data = match self {
            TaggedBody::Base64(d) => d,
            TaggedBody::String(d) => d,
        };
        data.as_ref()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CacheReqParts {
    #[serde(with = "crate::serde_cache::method")]
    pub method: Method,

    #[serde(with = "crate::serde_cache::header_map")]
    pub headers: HeaderMap<HeaderValue>,

    #[serde(with = "crate::serde_cache::query")]
    pub query: Option<PathAndQuery>,

    pub path: String,

    #[serde(with = "crate::serde_cache::bytes")]
    pub body: Bytes,
}

impl CacheReqParts {
    pub fn get_md5(&self) -> String {
        let method_bs: &[u8] = self.method.as_str().as_ref();
        let path: &[u8] = self.path.as_str().as_ref();
        let body: &[u8] = self.body.as_ref();
        let all = [method_bs, path, body].concat();
        let digest = md5::compute(&all);
        format!("{:x}", digest)
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct CacheConfig {
    pub match_headers: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CacheObject {
    pub request: CacheReqParts,

    #[serde(with = "crate::serde_cache::bytes")]
    pub response_body: Bytes,

    #[serde(with = "crate::serde_cache::header_map")]
    pub response_headers: HeaderMap<HeaderValue>,

    #[serde(with = "crate::serde_cache::status_code")]
    pub status_code: StatusCode,
    pub config: CacheConfig,
}

impl CacheObject {
    pub async fn find_by_req(
        req: &mut Request<Body>,
    ) -> (Option<Self>, CacheReqParts, String) {
        let method = req.method().clone();
        let headers = req.headers().clone();
        let query = req.uri().path_and_query().map(Clone::clone);
        let path = req.uri().path().to_string();
        let body = to_bytes(req.body_mut()).await.expect("read body failed");
        let req_parts = CacheReqParts {
            method,
            headers,
            query,
            path,
            body,
        };

        let md5 = req_parts.get_md5();
        let ret = GLOBAL_CACHE.0.get(&md5).map(|r| r.value().clone());
        (ret, req_parts, md5)
    }

    pub fn add_record_by_md5(self, md5: String) {
        GLOBAL_CACHE.0.insert(md5, self);
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Cache(pub DashMap<String, CacheObject>);

lazy_static! {
    pub static ref GLOBAL_CACHE: Cache = Default::default();
}

pub fn replace_global_cache(new_cache: &DashMap<String, CacheObject>) {
    let cache = &GLOBAL_CACHE.0;
    cache.clear();
    new_cache.iter().for_each(|e| {
        let (k, v) = e.pair();
        cache.insert(k.clone(), v.clone());
    });
}
