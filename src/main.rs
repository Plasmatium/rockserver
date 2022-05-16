mod cache;
mod cache_api;
mod config;
mod proxy;
mod serde_cache;

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    routing::{any, get, post},
    Router,
};
use tracing::info;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

use crate::{
    cache_api::{delete_cache_json, get_cache_json, post_cache_json},
    config::{Config, config_handler, G_CONFIG},
    proxy::proxy_handler,
};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_env("LOG_LEVEL").unwrap_or(EnvFilter::from("DEBUG"));
    FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish()
        .init();

    let fname = "config.yaml";
    let config = Config::from_file(fname);
    info!("config loaded, path: {fname}, content:\n{:?}", config);
    unsafe {
        G_CONFIG.apply(config);
    }
    let app = Router::new()
        .route("/rockserver/about", get(about_handler))
        .fallback(any(proxy_handler))
        .route(
            "/rockserver/cache.json",
            get(get_cache_json)
                .post(post_cache_json)
                .delete(delete_cache_json),
        )
        .route("/rockserver/config", post(config_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn about_handler() -> &'static str {
    return "rockserver for mock server and cache server";
}
