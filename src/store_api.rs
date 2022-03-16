use axum::{http, Json};
use hyper::StatusCode;

use crate::store::GLOBAL_STORE;

/**
 * API to operate on store
 *
 * GET  /rockserver/cache.json
 * POST /rockserver/cache.json
 */

pub async fn get_cache_json() -> (StatusCode, Json<String>) {
    let store = &GLOBAL_STORE.0;
    match serde_json::to_string_pretty(store) {
        Ok(body) => (StatusCode::OK, Json(body)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())),
    }
}

pub async fn post_cache_json(Json(cache_body): Json<String>) -> (http::StatusCode, String) {
    todo!()
}
