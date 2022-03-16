use anyhow::Result;
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

#[derive(Serialize, Deserialize, Clone)]
pub struct CacheParts {
    #[serde(with = "crate::serde_cache::method")]
    pub method: Method,

    #[serde(with = "crate::serde_cache::header_map")]
    pub headers: HeaderMap<HeaderValue>,

    #[serde(with = "crate::serde_cache::query")]
    pub query: Option<PathAndQuery>,

    pub path: String,

    pub body: TaggedBody,
}

impl CacheParts {
    pub fn get_md5(&self) -> Result<String> {
        let json_str = serde_json::to_string(self)?;
        let digest = md5::compute(json_str);
        Ok(format!("{:x}", digest))
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct CacheConfig {
    pub match_headers: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CacheObject {
    pub request: CacheParts,
    pub reponse: Option<CacheParts>,

    #[serde(with = "crate::serde_cache::status_code")]
    pub status_code: StatusCode,
    pub config: CacheConfig,
}

impl CacheObject {
    pub async fn find_by_req(req: &mut Request<Body>) -> Result<(Option<Self>, String)> {
        let method = req.method().clone();
        let headers = req.headers().clone();
        let query = req.uri().path_and_query().map(Clone::clone);
        let path = req.uri().path().to_string();
        let body: TaggedBody = (&to_bytes(req.body_mut()).await.expect("read body failed")).into();
        let req_parts = CacheParts {
            method,
            headers,
            query,
            path,
            body,
        };

        let md5 = req_parts.get_md5()?;
        let ret = GLOBAL_CACHE.get(&md5).map(|r| r.value().clone());
        Ok((ret, md5))
    }

    pub fn add_record_by_md5(self, md5: String) {
        GLOBAL_CACHE.insert(md5, self);
    }
}

lazy_static! {
    pub static ref GLOBAL_CACHE: DashMap<String, CacheObject> = Default::default();
}

#[cfg(test)]
mod tests {
    use crate::store::GLOBAL_STORE;
    use serde::{Deserialize, Serialize};
    use std::hash::{BuildHasher, Hasher};

    #[test]
    fn test_hash() {
        let store = &GLOBAL_STORE.0;
        let mut h = store.hasher().build_hasher();
        h.write(b"asfdf");
        let s = h.finish();
        println!("{s}")
    }

    #[derive(Deserialize, Serialize)]
    enum TTT {
        Base64(String),
        Str(String),
    }

    #[derive(Deserialize, Serialize)]
    struct AAA {
        f1: String,
        f2: TTT,
    }

    #[test]
    fn test_serde1() {
        let value = AAA {
            f1: "asdf".into(),
            f2: TTT::Base64("asdf".into()),
        };
        let ret = serde_json::to_string_pretty(&value).unwrap();
        println!("{ret}")
    }

    #[test]
    fn test_md5() {
        let a = b"some data\n";
        let digest = md5::compute(a);
        let d = format!("{:x}", digest);
        println!("{d}")
    }
}
