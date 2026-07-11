mod action;
mod app;
mod entity;
mod error;
mod helper;
mod router;
mod routes;

use crate::app::App;
use crate::router::Router;
use crate::routes::register_routes;
use minijinja::{path_loader, Environment};
use sea_orm::{ConnectOptions, Database};
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Attempt to load project-root/.env
    dotenvy::dotenv().unwrap();

    let router = Router::new(register_routes);

    let mut env = Environment::new();
    env.set_loader(path_loader(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resource/template"),
    ));

    let mut opt = ConnectOptions::new(env::var("DATABASE_URL").unwrap());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false) // disable SQLx logging
        .sqlx_logging_level(log::LevelFilter::Info);
    let db = Database::connect(opt).await.unwrap();
    db.get_schema_registry("bus_pattern_framework::entity::*")
        .sync(&db)
        .await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let app = Arc::new(App::new(router, addr, env, db).await);

    app::run(app).await
}
