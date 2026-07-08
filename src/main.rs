mod action;
mod app;
mod router;

use crate::action::pages::{ShowAbout, ShowJson, ShowNumberArray};
use crate::action::pages::{ShowHtml, ShowLanding};
use crate::app::App;
use crate::router::{Route, Router};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::rt::Executor;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use minijinja::{Environment, path_loader};
use sea_orm::{Database, DatabaseConnection};
use std::env;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

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

    let db: DatabaseConnection = Database::connect(env::var("DB_URL").unwrap())
        .await
        .unwrap();
    dbg!(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let app = App::new(router, addr, env).await;
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
