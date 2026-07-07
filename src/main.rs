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
use std::error::Error;
use std::fs;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
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

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let app = App::new(router, addr, env).await;
    let app = Arc::new(app);

    loop {
        let (stream, _) = app.listener().accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);
        // let router = Arc::clone(&self.router);
        let app = Arc::clone(&app);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `process` service
            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                // `service_fn` converts our function in a `Service`
                .serve_connection(
                    io,
                    service_fn(move |request: Request<Incoming>| {
                        // let router = Arc::clone(&router);
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
