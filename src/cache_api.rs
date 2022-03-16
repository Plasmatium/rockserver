use axum::{http, Json};
use dashmap::DashMap;
use hyper::StatusCode;

use crate::cache::{GLOBAL_CACHE, CacheObject, replace_global_cache};


/**
 * API to operate on store
 *
 * GET  /rockserver/cache.json
 * POST /rockserver/cache.json
 */

pub async fn get_cache_json() -> Json<&'static DashMap<String, CacheObject>> {
    let cache = &GLOBAL_CACHE.0;
    Json(cache)
}

pub async fn post_cache_json(Json(new_cache): Json<DashMap<String, CacheObject>>) -> http::StatusCode {
    replace_global_cache(&new_cache);
    StatusCode::ACCEPTED
}
