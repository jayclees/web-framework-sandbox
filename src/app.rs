use crate::router::Router;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use minijinja::Environment;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct App<'a> {
    router: Arc<Router>,
    listener: TcpListener,
    template: Environment<'a>,
}

impl<'a> App<'a> {
    pub async fn new(router: Router, addr: SocketAddr, env: Environment<'a>) -> App<'a> {
        App {
            router: Arc::new(router),
            listener: TcpListener::bind(addr).await.unwrap(),
            template: env,
        }
    }

    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }

    pub fn template(&self) -> &Environment<'a> {
        &self.template
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
