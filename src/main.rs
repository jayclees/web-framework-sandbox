mod action;
mod app;
mod entity;
mod router;
mod error;

use crate::action::pages::{ShowAbout, ShowDatabaseModel, ShowErrorPage, ShowJson, ShowNumberArray};
use crate::action::pages::{ShowHtml, ShowLanding};
use crate::app::App;
use crate::router::{Route, Router};
use hyper::rt::Executor;
use minijinja::{Environment, path_loader};
use sea_orm::{ConnectOptions, Database};
use std::env;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Attempt to load project-root/.env
    dotenvy::dotenv().unwrap();

    let mut router = Router::new();

    router.add(Route::new("/", Box::new(ShowLanding)));
    router.add(Route::new("/about", Box::new(ShowAbout)));
    router.add(Route::new("/json-array", Box::new(ShowNumberArray)));
    router.add(Route::new("/json", Box::new(ShowJson)));
    router.add(Route::new("/html", Box::new(ShowHtml)));
    router.add(Route::new("/db-user", Box::new(ShowDatabaseModel)));
    router.add(Route::new("/error", Box::new(ShowErrorPage)));

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

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let app = Arc::new(App::new(router, addr, env, db).await);

    app.run().await
}

/// Future executor that utilises `tokio` threads.
#[non_exhaustive]
#[derive(Default, Debug, Clone)]
pub struct TokioExecutor;

impl TokioExecutor {
    pub fn new() -> Self {
        Self {}
    }
}

impl<Fut> Executor<Fut> for TokioExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        tokio::spawn(fut);
    }
}
