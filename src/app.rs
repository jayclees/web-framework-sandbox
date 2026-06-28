use crate::TokioExecutor;
use crate::router::Router;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct App {
    router: Arc<Router>,
    listener: Option<TcpListener>,
}

impl App {
    pub fn new(router: Router) -> App {
        App {
            router: Arc::new(router),
            listener: None,
        }
    }

    pub async fn run(
        &mut self,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.listener = Some(TcpListener::bind(addr).await.unwrap());
        loop {
            let (stream, _) = self.listener.as_ref().unwrap().accept().await?;

            // Use an adapter to access something implementing `tokio::io` traits as if they implement
            // `hyper::rt` IO traits.
            let io = TokioIo::new(stream);
            let router = Arc::clone(&self.router);

            // Spawn a tokio task to serve multiple connections concurrently
            tokio::task::spawn(async move {
                // Finally, we bind the incoming connection to our `process` service
                if let Err(err) = auto::Builder::new(TokioExecutor::new())
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(
                        io,
                        service_fn(move |request: Request<Incoming>| {
                            let router = Arc::clone(&router);

                            async move {
                                router.dispatch(request).await.unwrap_or_else(|| {
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
}
