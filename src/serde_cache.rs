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

pub mod query {
    use std::str::FromStr;

    use axum::http::uri::PathAndQuery;
    use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PathAndQuery>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Option::<String>::deserialize(deserializer)?;
        if raw.is_none() {
            return Ok(None)
        }
        let raw = raw.unwrap();
        let query = PathAndQuery::from_str(&raw).map_err(D::Error::custom)?;
        Ok(Some(query))
    }

    pub fn serialize<S>(value: &Option<PathAndQuery>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.as_ref().map(ToString::to_string).serialize(serializer)
    }
}