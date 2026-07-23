mod action;
mod entity;
mod routes;

use crate::routes::register_routes;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use sturdy::app::{App, Env};
use sturdy::cli::process_args;
use sturdy::database::db;
use sturdy::error::register_panic_hook;
use sturdy::routing::router::Router;
use sturdy::support::logger::Logger;
use sturdy::template::reloader;

struct AppState {
    //
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenvy::dotenv()?;
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let logger = Logger::new(root.clone());
    register_panic_hook(logger.clone());

    let (host, port, vite_url) = process_args();
    let state = AppState {};
    let app = App::new(
        Router::new(register_routes),
        format!("{host}:{port}"),
        reloader(root.clone()),
        db().await?,
        logger,
        Env::new(env::var("APP_ENV")?, true, Some(vite_url)),
        Box::new(state),
    )
    .await;

    sturdy::app::run(Arc::new(app)).await
}
