mod action;
mod entity;
mod routes;

use crate::routes::register_routes;
use framework::app::App;
use framework::error::register_panic_hook;
use framework::routing::router::Router;
use minijinja::{path_loader, Environment};
use minijinja_autoreload::AutoReloader;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    register_panic_hook(root.clone());
    dotenvy::dotenv()?;

    // Attempt to load project-root/.env

    let router = Router::new(register_routes);
    let template_reloader = reloader();
    let db = db().await?;
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let app = App::new(router, addr, template_reloader, db).await;
    let app = Arc::new(app);

    framework::app::run(app).await
}

async fn db() -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let mut opt = ConnectOptions::new(env::var("DATABASE_URL")?);
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
    Ok(db)
}

fn reloader() -> AutoReloader {
    // If DISABLE_AUTORELOAD is set, then the path tracking is disabled.
    let disable_autoreload = env::var("DISABLE_AUTORELOAD").as_deref() == Ok("1");

    // If FAST_AUTORELOAD is set, then fast reloading is enabled.
    let fast_autoreload = env::var("FAST_AUTORELOAD").as_deref() == Ok("1");

    // The closure is invoked every time the environment is outdated to
    // recreate it.
    AutoReloader::new(move |notifier| {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resource/template");
        let mut env = Environment::new();
        env.set_loader(path_loader(&template_path));

        if fast_autoreload {
            notifier.set_fast_reload(true);
        }

        // if watch_path is never called, no fs watcher is created
        if !disable_autoreload {
            notifier.watch_path(&template_path, true);
        }
        Ok(env)
    })
}
