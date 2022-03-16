use std::sync::Arc;

use axum::{body::Bytes, extract::Extension, http::HeaderValue, response::Response};
use hyper::{Body, HeaderMap, Request, StatusCode};
use reqwest::Client;

use crate::{
    cache::{CacheObject, CacheReqParts, TaggedBody},
    config::Config,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub async fn proxy_handler(
    Extension(config): Extension<Arc<Config>>,
    mut req: Request<Body>,
) -> Response<Body> {
    let result = CacheObject::find_by_req(&mut req).await;
    if let Err(e) = result {
        return make_resp(
            HeaderMap::default(),
            StatusCode::INTERNAL_SERVER_ERROR,
            Bytes::from(e.to_string()),
        );
    }

    let (cached, req_parts, md5) = result.unwrap();
    if let Some(CacheObject {
        ref response_body,
        response_headers,
        status_code,
        ..
    }) = cached
    {
        let body: Bytes = response_body.into();
        return make_resp(response_headers, status_code, body);
    }

    let CacheReqParts {
        ref headers,
        ref body,
        ..
    } = req_parts;
    let mut uri_parts = req.uri().clone().into_parts();
    let Config { proxy, .. } = config.as_ref();
    uri_parts.scheme = Some(proxy.scheme.as_str().try_into().unwrap());
    uri_parts.authority = Some(proxy.authority.as_str().try_into().unwrap());
    *req.uri_mut() = uri_parts.try_into().unwrap();
    let mut headers = headers.clone();
    headers.remove("host");
    let url: String = req.uri().to_string();
    let body: Bytes = body.into();
    let ret_resp = CLIENT
        .request(req.method().clone(), url)
        .body(body)
        .headers(headers)
        .send()
        .await;
    if let Err(e) = ret_resp {
        return make_resp(
            HeaderMap::default(),
            StatusCode::INTERNAL_SERVER_ERROR,
            Bytes::from(e.to_string()),
        );
    }
    let ret_resp = ret_resp.unwrap();
    let status_code = ret_resp.status();
    let mut headers = ret_resp.headers().clone();
    headers.insert(
        "x-rockserver-id",
        HeaderValue::from_str(&md5).expect("md5 contains non ascii code"),
    );
    let resp_bs = ret_resp
        .bytes()
        .await
        .expect("read resp body failed");
    let body: TaggedBody = (&resp_bs).into();

    let cached = CacheObject {
        request: req_parts,
        response_body: body,
        response_headers: headers.clone(),
        status_code,
        config: Default::default(),
    };
    cached.add_record_by_md5(md5);

    headers.insert("x-rockserver", HeaderValue::from_static("miss"));
    make_resp(headers, status_code, resp_bs)

    //     headers.insert("x-rockserver", HeaderValue::from_static("miss"));
    //     make_resp(headers, status_code, resp_bs)
    // let key = CacheKey::read_from_req(&mut req).await;
    // let store = &GLOBAL_STORE.0;
    // if let Some(entry) = store.get(&key) {
    //     let CacheValue {
    //         mut headers,
    //         status_code,
    //         body_bs,
    //         ..
    //     } = entry.value().clone();
    //     headers.insert("x-rockserver", HeaderValue::from_static("hit"));
    //     make_resp(headers, status_code, body_bs)
    // } else {
    //     let mut uri_parts = req.uri().clone().into_parts();
    //     let Config { proxy, .. } = config.as_ref();
    //     uri_parts.scheme = Some(proxy.scheme.as_str().try_into().unwrap());
    //     uri_parts.authority = Some(proxy.authority.as_str().try_into().unwrap());
    //     *req.uri_mut() = uri_parts.try_into().unwrap();

    //     let body = key.get_body();
    //     debug!("body length: {}", body.len());
    //     let mut headers = req.headers().clone();
    //     headers.remove("host");
    //     let url: String = req.uri().to_string();
    //     let ret_resp = CLIENT
    //         .request(req.method().clone(), url)
    //         .body(body.clone())
    //         .headers(headers)
    //         .send()
    //         .await
    //         .unwrap();

    //     let status_code = ret_resp.status();
    //     let mut headers = ret_resp.headers().clone();
    //     let resp_bs = ret_resp.bytes().await.unwrap();
    //     let val = CacheValue {
    //         headers: headers.clone(),
    //         status_code,
    //         body_bs: resp_bs.clone(),
    //         cache_config: Default::default(),
    //     };
    //     store.insert(key, val);
    //     headers.insert("x-rockserver", HeaderValue::from_static("miss"));
    //     make_resp(headers, status_code, resp_bs)
    // }
}

fn make_resp(headers: HeaderMap, status_code: StatusCode, body: Bytes) -> Response<Body> {
    let resp_body = Body::from(body);
    let mut resp = Response::new(resp_body);
    *resp.headers_mut() = headers;
    *resp.status_mut() = status_code;
    resp
}
