use hyper::body::Incoming;
use hyper::{Method, Request};
use crate::error::HttpError;
use crate::routing::action::Action;
use crate::routing::route::Route;

#[derive(Debug)]
pub struct Router {
    pub routes: Vec<Route>,
}

impl Router {
    pub fn new(register_routes: fn(&mut Router)) -> Router {
        let mut router = Router { routes: Vec::new() };

        register_routes(&mut router);

        router
    }

    pub fn add<A: Action + 'static>(
        &mut self,
        method: Method,
        path: &'static str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        let mut route = Route::new(method, path, action);

        if let Some(modifier) = modifier {
            route = modifier(route);
        }

        self.routes.push(route);

        self
    }

    pub fn get<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
    ) -> &mut Router {
        self.add(Method::GET, path, action, None);

        self
    }

    pub fn getm<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::GET, path, action, Some(modifier));

        self
    }

    pub fn post<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::POST, path, action, modifier);

        self
    }

    pub fn patch<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::PATCH, path, action, modifier);

        self
    }

    pub fn put<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::PUT, path, action, modifier);

        self
    }

    pub fn delete<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::DELETE, path, action, modifier);

        self
    }

    pub fn head<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::HEAD, path, action, modifier);

        self
    }

    pub fn connect<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::CONNECT, path, action, modifier);

        self
    }

    pub fn options<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::OPTIONS, path, action, modifier);

        self
    }

    pub fn trace<A: Action + 'static>(
        &mut self,
        path: &'static str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::TRACE, path, action, modifier);

        self
    }

    pub fn resolve(&self, request: &Request<Incoming>) -> Result<Option<&Route>, HttpError> {
        self.resolve_inner(request.uri().path(), request.method())
    }

    /// This lets us test without requiring a Request<Incoming> instance
    pub fn resolve_inner(&self, path: &str, method: &Method) -> Result<Option<&Route>, HttpError> {
        for route in &self.routes {
            if route.matches(path) {
                return if method != route.get_method() {
                    Err(HttpError::new(405, "Method not allowed".to_string()))
                } else {
                    Ok(Some(route))
                };
            }
        }

        Ok(None)
    }
}
