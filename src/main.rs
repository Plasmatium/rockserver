mod config;
mod proxy;
mod store;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::Extension,
    routing::{any, get},
    Router,
};
use tracing::info;

use crate::{config::Config, proxy::proxy_handler};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let fname = "config.yaml";
    let config = Config::from_file(fname);
    info!("config loaded, path: {fname}, content:\n{:?}", config);
    let app = Router::new()
        .route("/", get(index_handler))
        .fallback(any(proxy_handler))
        .layer(Extension(Arc::new(config)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn index_handler() -> &'static str {
    return "hello world";
}
