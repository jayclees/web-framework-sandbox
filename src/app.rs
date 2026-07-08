use crate::TokioExecutor;
use crate::router::Router;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use minijinja::Environment;
use sea_orm::DatabaseConnection;
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct App {
    router: Arc<Router>,
    listener: TcpListener,
    template: Environment<'static>,
    db: Option<DatabaseConnection>,
}

impl App {
    pub async fn new(
        router: Router,
        addr: SocketAddr,
        env: Environment<'static>,
        db: DatabaseConnection,
    ) -> App {
        App {
            router: Arc::new(router),
            listener: TcpListener::bind(addr).await.unwrap(),
            template: env,
            db: Some(db),
        }
    }

    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }

    pub fn template(&self) -> &Environment<'static> {
        &self.template
    }

    pub fn db(&self) -> Option<&DatabaseConnection> {
        self.db.as_ref()
    }

    pub async fn run(self: Arc<Self>) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            let (stream, _) = self.listener().accept().await?;
            let io = TokioIo::new(stream);
            let app = Arc::clone(&self);

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

    pub async fn dispatch(
        &self,
        request: Request<Incoming>,
    ) -> Option<Result<Response<Full<Bytes>>, Infallible>> {
        for route in &self.router.routes {
            if route.path() == request.uri().path() {
                let result = Some(route.action().handle(&self).await.to_response());

                route.action().log().await;

                return result;
            }
        }
        None
    }
}
