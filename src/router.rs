use crate::action::Action;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::str::Split;
use std::sync::LazyLock;
use hyper::body::Incoming;
use hyper::{Method, Request};
use crate::error::HttpError;

static REG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^_][a-zA-Z0-9_]*[a-zA-Z0-9]}").unwrap());

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

    pub fn add(&mut self, route: Route) -> &mut Router {
        self.routes.push(route);
        self
    }

    pub fn resolve(&self, request: Request<Incoming>) -> Result<Option<&Route>, HttpError> {
        // todo: handle constrained route parameter paths (potential wildcards)
        for route in &self.routes {
            if route.matches(request.uri().path()) {
                return if request.method() != route.method {
                    Err(HttpError::new(405, "Method not allowed".to_string()))
                } else {
                    Ok(Some(route))
                }
            }
        }

        Ok(None)
    }
}

type ActionType = Box<dyn Action + Send + Sync>;

#[derive(Debug)]
pub struct Route {
    // todo implement route names
    name: Option<&'static str>,
    // todo use http::Method(Inner) enum
    method: Method,
    path: &'static str,
    segments: Vec<RouteSegment<'static>>,
    action: ActionType,
    filter: Option<()>,
}

impl Route {
    pub fn new(method: Method, path: &'static str, action: ActionType) -> Route {
        // let path = if !path.starts_with("/") {
        //     let t: &'static str = format!("/{path}").as_str();
        //     t
        // } else {
        //     path
        // };
        Route {
            name: None,
            method,
            path,
            segments: process_segments(split_segments(path)),
            action,
            filter: None,
        }
    }

    pub fn get(path: &'static str, action: ActionType) -> Route {
        Self::new(Method::GET, path, action)
    }

    pub fn post(path: &'static str, action: ActionType) -> Route {
        Self::new(Method::POST, path, action)
    }

    pub fn patch(path: &'static str, action: ActionType) -> Route {
        Self::new(Method::PATCH, path, action)
    }

    pub fn put(path: &'static str, action: ActionType) -> Route {
        Self::new(Method::PUT, path, action)
    }

    pub fn delete(path: &'static str, action: ActionType) -> Route {
        Self::new(Method::DELETE, path, action)
    }

    pub fn head(path: &'static str, action: ActionType) -> Route {
        Self::new(Method::HEAD, path, action)
    }

    pub fn connect(path: &'static str, action: ActionType) -> Route {
        Self::new(Method::CONNECT, path, action)
    }

    pub fn options(path: &'static str, action: ActionType) -> Route {
        Self::new(Method::OPTIONS, path, action)
    }

    pub fn trace(path: &'static str, action: ActionType) -> Route {
        Self::new(Method::TRACE, path, action)
    }

    pub fn path(&self) -> &'static str {
        self.path
    }

    pub fn action(&self) -> &ActionType {
        &self.action
    }

    pub fn name(mut self, name: &'static str) -> Route {
        self.name = Some(name);
        self
    }

    pub fn get_name(&self) -> Option<&'static str> {
        self.name
    }

    pub fn constrain(&self, parameter: &str, pattern: &str) -> &Self {
        todo!("implement constraints for route parameter");
        self
    }

    pub fn matches(&self, path: &str) -> bool {
        let variable_segment_count = self
            .segments
            .iter()
            .filter(|seg| match seg {
                RouteSegment::String(_) => true,
                _ => false,
            })
            .collect::<Vec<&RouteSegment>>()
            .len();
        if variable_segment_count == self.segments.len() {
            // if all segments are strings, just do string equality check
            return self.path == path;
        }

        let req_segs = split_segments(path).collect::<Vec<&str>>();
        let rou_segs = &self.segments;
        let mut matches = true;
        let mut step = 0;

        // Loop over both request segments and route segments
        // and process each segment. todo check for wildcard variables
        loop {
            let req_seg = req_segs.iter().nth(step);
            let rou_seg = rou_segs.iter().nth(step);

            if let None = req_seg && let None = rou_seg {
                // both ran out at same time
                // break out of loop and return the current value of matches
                return matches;
            }

            // One ran out before the other. Segment counts do not match. Return false.
            if let Some(_) = req_seg && let None = rou_seg {
                return false;
            }

            // One ran out before the other. Segment counts do not match. Return false.
            if let None = req_seg && let Some(_) = rou_seg {
                return false;
            }

            if let Some(req_seg) = req_seg
                && let Some(rou_seg) = rou_seg
            {
                matches = match rou_seg {
                    RouteSegment::String(rou_seg) => rou_seg == req_seg,
                    RouteSegment::Variable {
                        handle: _,
                        matches: _,
                    } => {
                        // todo check if variable has regex constraint
                        // it'll always be true unless there is a constraint
                        true
                    }
                    RouteSegment::ModelId(_) => true,
                };

                if !matches {
                    break;
                }
            }

            step += 1;
        }

        matches
    }
}

#[derive(Debug)]
enum RouteSegment<'a> {
    String(&'a str),
    Variable {
        handle: &'a str,
        matches: Vec<&'a str>,
    },
    ModelId(&'a str),
    // regular string path
    // model string name
}

impl<'a> Display for RouteSegment<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteSegment::String(str) => write!(
                f,
                "RouteSegment::String({})",
                if *str == "" { "\"\"" } else { str }
            ),
            RouteSegment::Variable { handle, matches } => {
                write!(f, "RouteSegment::Variable({handle}, {matches:?})")
            }
            RouteSegment::ModelId(str) => write!(f, "RouteSegment::ModelId({str})"),
        }
    }
}

/// Since we want to split the route definition path and the request
/// instance path the same way we will extract it into a helper fn
fn split_segments<'a>(path: &'a str) -> Split<'a, &'static str> {
    path.split("/")
}

fn process_segments<'a>(segments: Split<'a, &'static str>) -> Vec<RouteSegment<'a>> {
    let mut result = Vec::new();

    for segment in segments.into_iter() {
        result.push(segment_type(segment));
    }

    result
}

fn segment_type(segment: &str) -> RouteSegment<'_> {
    let matches: Vec<&str> = REG.find_iter(segment).map(|m| m.as_str()).collect();

    if matches.len() == 0 {
        return RouteSegment::String(segment);
    }

    // E.g. example.com/user/{user}
    //                       ^^^^^^
    RouteSegment::Variable {
        handle: segment,
        matches,
    }
}

fn is_variable(segment: &str) -> bool {
    todo!()
}

fn is_model(segment: &str) -> bool {
    todo!()
}
