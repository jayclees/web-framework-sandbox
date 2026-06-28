use crate::action::Action;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use std::convert::Infallible;

pub struct Router {
    pub routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Router {
        Router { routes: Vec::new() }
    }

    pub fn add(&mut self, route: Route) -> &mut Router {
        self.routes.push(route);
        self
    }

    pub async fn dispatch(
        &self,
        request: Request<Incoming>,
    ) -> Option<Result<Response<Full<Bytes>>, Infallible>> {
        for route in &self.routes {
            if route.path == request.uri().path() {
                let result = Some(route.action.handle().await);

                route.action.log().await;

                return result;
            }
        }
        None
    }
}

type ActionType = Box<dyn Action + Send + Sync>;

pub struct Route {
    path: &'static str,
    action: ActionType,
}

impl Route {
    pub fn new(path: &'static str, action: ActionType) -> Route {
        Route { path, action }
    }
}
