use crate::error::HttpError;
use crate::routing::router::Router;
use futures::FutureExt;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::rt::Executor;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use minijinja_autoreload::AutoReloader;
use sea_orm::DatabaseConnection;
use serde_json::json;
use std::error::Error;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Debug)]
pub struct Env {
    env: String,
    debug: bool,
}

impl Env {
    // todo implement getters/setters
    // todo load .env file
}

pub struct App {
    router: Arc<Router>,
    listener: TcpListener,
    template: AutoReloader,
    db: Option<DatabaseConnection>,
    env: Env,
}

impl App {
    pub async fn new(
        router: Router,
        addr: SocketAddr,
        template: AutoReloader,
        db: DatabaseConnection,
    ) -> App {
        App {
            router: Arc::new(router),
            listener: TcpListener::bind(addr).await.unwrap(),
            template,
            db: Some(db),
            env: Env {
                env: "production".to_string(),
                debug: true,
            },
        }
    }

    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }

    pub fn template(&self) -> &AutoReloader {
        &self.template
    }

    pub fn db(&self) -> Option<&DatabaseConnection> {
        self.db.as_ref()
    }

    pub async fn dispatch(
        &self,
        request: Request<Incoming>,
    ) -> Option<Result<Response<Full<Bytes>>, HttpError>> {
        let result = &self.router.resolve(&request);

        match result {
            Ok(route) => match route {
                Some(route) => match route.action().handle(&self, request).await {
                    Ok(result) => {
                        route.action().log().await;
                        Some(result.to_response())
                    }
                    Err(e) => Some(Err(e)),
                },
                None => Some(Err(HttpError::new(404, "Page not found".to_string()))),
            },
            Err(error) => {
                // todo figure out why the error param is passed by reference
                Some(Err(error.clone()))
            }
        }
    }

    fn error(&self, error: &HttpError, json: bool) -> Result<Response<Full<Bytes>>, HttpError> {
        let mut builder = Response::builder().status(error.code());

        // todo check if error templates available, if so, return the error template
        // todo check if app is local/debug is enabled, send stack trace to browser if so

        let content = if json {
            builder = builder.header("Content-Type", "application/json");
            json!({
                "code": error.code(),
                "message": error.message(),
            })
            .to_string()
        } else {
            self.template
                .acquire_env()
                .unwrap()
                .get_template("errors/default.html")
                .unwrap()
                // todo WARNING: be careful what we send to client here.
                .render(error)
                .unwrap()
        };

        Ok(builder.body(Full::new(Bytes::from(content))).unwrap())
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

pub async fn run(app: Arc<App>) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        let (stream, _) = app.listener().accept().await?;
        let io = TokioIo::new(stream);
        let app = Arc::clone(&app);

        tokio::task::spawn(async move {
            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(
                    io,
                    service_fn(move |request: Request<Incoming>| {
                        handle_request(Arc::clone(&app), request)
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    app: Arc<App>,
    request: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, HttpError> {
    let wants_json = if let Some(accepts) = request.headers().get("Accept") {
        Some(accepts == "application/json")
    } else {
        None
    };

    // Catch panics within app.dispatch()
    match AssertUnwindSafe(app.dispatch(request)).catch_unwind().await {
        Ok(option) => {
            match option {
                Some(result) => {
                    if let Err(err) = &result {
                        if wants_json.unwrap_or(false) {
                            return app.error(err, true);
                        }

                        return app.error(err, false);
                    }

                    result
                }
                None => {
                    // if response wants JSON or is api route, return JSON
                    // else, check config for error templates, return that
                    if wants_json.unwrap_or(false) {
                        return app
                            .error(&HttpError::new(404, "Endpoint not found.".to_owned()), true);
                    }

                    app.error(&HttpError::new(404, "Page not found.".to_owned()), false)
                }
            }
        }
        Err(error) => {
            // Server error. Send details if app is local and debug is enabled
            let msg = if app.env.env == "local" && app.env.debug {
                if let Some(msg) = error.downcast_ref::<&str>() {
                    *msg
                } else if let Some(msg) = error.downcast_ref::<String>() {
                    msg
                } else {
                    "Unknown panic."
                }
            } else {
                "Something went wrong."
            };

            app.error(&HttpError::new(500, msg.to_string()), false)
        }
    }
}
