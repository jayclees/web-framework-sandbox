mod action;
mod app;
mod router;
mod entity;

use crate::action::pages::{ShowAbout, ShowJson, ShowNumberArray};
use crate::action::pages::{ShowHtml, ShowLanding};
use crate::app::App;
use crate::router::{Route, Router};
use crate::entity::user::Entity as User;
use crate::entity::user;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::rt::Executor;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use minijinja::{Environment, path_loader};
use sea_orm::{ConnectOptions, Database, EntityTrait};
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
    let user: Option<user::Model> = User::find_by_id(1).one(&db).await?;
    dbg!(user);
    std::process::exit(1);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let app = App::new(router, addr, env, db).await;
    let app = Arc::new(app);

    loop {
        let (stream, _) = app.listener().accept().await?;
        let io = TokioIo::new(stream);
        let app = Arc::clone(&app);

        tokio::task::spawn(async move {
            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(
                    io,
                    service_fn(move |request: Request<Incoming>| {
                        let app = Arc::clone(&app);

                        async move {
                            app.dispatch(request).await.unwrap_or_else(|| {
                                Ok(Response::builder()
                                    .status(404)
                                    .body(Full::new(Bytes::from("Not Found")))
                                    .unwrap())
                            })
                        }
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
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
