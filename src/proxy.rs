use axum::{body::Bytes, http::HeaderValue, response::Response};
use hyper::{header::CONTENT_LENGTH, Body, HeaderMap, Request, StatusCode};
use reqwest::Client;

use crate::{
    cache::{CacheObject, CacheReqParts},
    config::{Config, G_CONFIG},
};
use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

/// Proxy the resquest and store the response. The stored response will be sent immedietly next time
/// if the request matched.
/// step 0. if not enabled, then just proxy pass
/// step 1. construct the request parts and caculated the md5 for store key
/// step 2. return if request matched
/// step 3. else proxy the request to remote
/// step 4. store the response from the remote
/// step 5. send response back to client
pub async fn proxy_handler(
    mut req: Request<Body>,
) -> Response<Body> {
    let config: &Config;
    unsafe{ config = &G_CONFIG }
    // step 1. construct the request parts and caculated the md5 for store key
    let (cached, req_parts, md5) = CacheObject::find_by_req(&mut req).await;
    if config.enabled == Some(true) {
        if let Some(CacheObject {
            response_body,
            mut response_headers,
            status_code,
            ..
        }) = cached
        {
            // step 2. return if request matched
            response_headers.insert("x-rockserver", HeaderValue::from_static("hit"));
            return make_resp(response_headers, status_code, response_body);
        }
    }

    // step 3. else proxy the request to remote
    let CacheReqParts {
        ref headers,
        ref body,
        ..
    } = req_parts;
    let mut uri_parts = req.uri().clone().into_parts();
    let uri = config.get_uri().expect("invalid proxy uri");
    uri_parts.scheme = uri.scheme().cloned();
    uri_parts.authority = uri.authority().cloned();
    *req.uri_mut() = uri_parts.try_into().unwrap();
    let mut headers = headers.clone();
    headers.remove("host");
    let url: String = req.uri().to_string();
    let ret_resp = CLIENT
        .request(req.method().clone(), url)
        .body(body.clone())
        .headers(headers.clone())
        .send()
        .await;
    if let Err(e) = ret_resp {
        return make_resp(
            HeaderMap::default(),
            StatusCode::INTERNAL_SERVER_ERROR,
            Bytes::from(e.to_string()),
        );
    }

    // step 4. store the response from the remote
    let ret_resp = ret_resp.unwrap();
    let status_code = ret_resp.status();
    let mut headers = ret_resp.headers().clone();
    let body = ret_resp.bytes().await.expect("read resp body failed");
    let mut status_code_threshold = StatusCode::from_u16(599).unwrap();
    if let Some(threshold) = config.status_code_threshold {
        if let Ok(valid_status_code) = StatusCode::from_u16(threshold) {
            status_code_threshold = valid_status_code;
        }
    }
    if status_code < status_code_threshold {
        headers.insert(
            "x-rockserver-id",
            HeaderValue::from_str(&md5).expect("md5 contains non ascii code"),
        );

        headers.remove(CONTENT_LENGTH);

        let cached = CacheObject {
            request: req_parts,
            response_body: body.clone(),
            response_headers: headers.clone(),
            status_code,
            config: Default::default(),
        };
        cached.add_record_by_md5(md5);
    }

    // step 5. send response back to client
    headers.insert("x-rockserver", HeaderValue::from_static("miss"));
    make_resp(headers, status_code, body)
}

fn make_resp(headers: HeaderMap, status_code: StatusCode, body: Bytes) -> Response<Body> {
    let resp_body = Body::from(body);
    let mut resp = Response::new(resp_body);
    *resp.headers_mut() = headers;
    *resp.status_mut() = status_code;
    resp
}
