mod config;
mod proxy;
mod store;
mod store_api;
mod cache;
mod serde_cache;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::Extension,
    routing::{any, get},
    Router,
};
use tracing::info;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

use crate::{
    config::Config,
    proxy::proxy_handler,
    store_api::{get_cache_json, post_cache_json},
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
    let app = Router::new()
        .route("/about", get(about_handler))
        .fallback(any(proxy_handler))
        .layer(Extension(Arc::new(config)))
        .route(
            "/rockserver/config.json",
            get(get_cache_json).post(post_cache_json),
        );

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
