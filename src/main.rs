mod action;
mod app;
mod router;

use crate::action::pages::ShowAbout;
use crate::action::pages::ShowLanding;
use crate::app::App;
use crate::router::{Route, Router};
use hyper::rt::Executor;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;

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

    App::new(router)
        .run(SocketAddr::from(([127, 0, 0, 1], 3000)))
        .await
}
