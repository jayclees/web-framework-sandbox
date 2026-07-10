use crate::error::HttpError;
use crate::router::Router;
use crate::TokioExecutor;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use minijinja::Environment;
use sea_orm::DatabaseConnection;
use serde_json::json;
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

    pub async fn dispatch(
        &self,
        request: &Request<Incoming>,
    ) -> Option<Result<Response<Full<Bytes>>, HttpError>> {
        let route = &self.router.resolve(request.uri().path())?;

        match route.action().handle(&self).await {
            Ok(result) => {
                route.action().log().await;

                Some(result.to_response())
            }
            Err(e) => Some(Err(e)),
        }
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

    let option = app.dispatch(&request).await;
    match option {
        Some(result) => {
            if let Err(err) = &result {
                if wants_json.unwrap_or(false) {
                    let json = json!({
                        "code": err.code(),
                        "message": err.message(),
                    });
                    return error_response(err.code(), json.to_string(), true);
                }

                return error_response(err.code(), err.message(), false);
            }

            result
        }
        None => {
            // if response wants JSON or is api route, return JSON
            // else, check config for error templates, return that
            if wants_json.unwrap_or(false) {
                let json = json!({
                    "code": 404,
                    "message": "Endpoint not found."
                });
                return error_response(404, json.to_string(), true);
            }

            error_response(404, "Page not found.".to_string(), false)
        }
    }
}

fn error_response(
    code: u16,
    message: String,
    json: bool,
) -> Result<Response<Full<Bytes>>, HttpError> {
    let mut builder = Response::builder().status(code);

    if json {
        builder = builder.header("Content-Type", "application/json");
    }
    // todo check if error templates available, if so, return the error template
    // todo check if app is local/debug is enabled, send stack trace to browser if so

    Ok(builder.body(Full::new(Bytes::from(message))).unwrap())
}
