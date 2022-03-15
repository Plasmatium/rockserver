use axum::body::Bytes;

use dashmap::DashMap;
use hyper::{body::to_bytes, Body, HeaderMap, Method, Request, StatusCode};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

// CacheKey, (method, path, body)
#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey(
    #[serde(with = "serde_store::method")]
    Method,

    String,

    #[serde(with = "serde_store::body_bs")]
    Bytes
);

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
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CacheValue {
    #[serde(with = "serde_store::header_map")]
    pub headers: HeaderMap,

    #[serde(with = "serde_store::status_code")]
    pub status_code: StatusCode,

    #[serde(with = "serde_store::body_bs")]
    pub body_bs: Bytes,

    pub cache_config: CacheConfig,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CacheConfig {
    pub match_headers: Vec<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Store(pub DashMap<CacheKey, CacheValue>);

impl Store {}

lazy_static! {
    pub static ref GLOBAL_STORE: Store = Store(Default::default());
}

pub mod serde_store {
    pub mod header_map {
        use std::collections::HashMap;

        use axum::http::HeaderValue;
        use hyper::HeaderMap;
        use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

        pub fn deserialize<'de, D>(deserializer: D) -> Result<HeaderMap<HeaderValue>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let h = HashMap::<String, String>::deserialize(deserializer)?;
            let headermap = HeaderMap::<HeaderValue>::try_from(&h).map_err(D::Error::custom)?;
            return Ok(headermap);
        }

        pub fn serialize<S>(
            value: &HeaderMap<HeaderValue>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let header_hashmap: HashMap<String, String> = value
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        v.to_str().expect("invalid header value").to_string(),
                    )
                })
                .collect();

            header_hashmap.serialize(serializer)
        }
    }

    pub mod status_code {
        use hyper::StatusCode;
        use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

        pub fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
        where
            D: Deserializer<'de>,
        {
            let raw = u16::deserialize(deserializer)?;
            let status_code = StatusCode::from_u16(raw).map_err(D::Error::custom)?;
            Ok(status_code)
        }

        pub fn serialize<S>(value: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            value.as_u16().serialize(serializer)
        }
    }

    pub mod body_bs {
        use bytes::Bytes;
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
        where
            D: Deserializer<'de>,
        {
            let body_str = String::deserialize(deserializer)?;
            Ok(Bytes::from(body_str))
        }

        pub fn serialize<S>(value: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let bs = value.as_ref().to_vec();
            String::from_utf8(bs)
                .unwrap_or("invalid utf8 bytes".to_string())
                .serialize(serializer)
        }
    }

    pub mod method {
        use std::str::FromStr;

        use hyper::Method;
        use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Method, D::Error>
        where
            D: Deserializer<'de>,
        {
            let raw = String::deserialize(deserializer)?;
            Method::from_str(&raw).map_err(D::Error::custom)
        }

        pub fn serialize<S>(value: &Method, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            value.to_string().serialize(serializer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_serde() {
        let mut headers = HeaderMap::<HeaderValue>::default();
        headers.insert("key", HeaderValue::from_static("src"));

        let cv = CacheValue {
            headers,
            status_code: StatusCode::from_u16(200).unwrap(),
            body_bs: Bytes::from_static(b"bytes"),
            cache_config: Default::default(),
        };

        let s = serde_json::to_string_pretty(&cv).unwrap();
        println!("{s}");

        let cv2: CacheValue = serde_json::from_str(&s).unwrap();
        println!("{:#?}", cv2);
        assert_eq!(cv, cv2);
    }
}
