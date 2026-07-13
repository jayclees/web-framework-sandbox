use std::borrow::Cow;
use crate::action::Action;
use crate::error::HttpError;
use hyper::body::Incoming;
use hyper::{Method, Request};
use regex::Regex;
use std::ops::Range;
use std::str::Split;
use std::sync::LazyLock;

static REG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^_][a-zA-Z0-9_]*[a-zA-Z0-9]}").unwrap());
static VAR_DELIMITER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\A\{)|(})\z").unwrap());

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
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::GET, path, action, modifier);

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
        // todo: handle constrained route parameter paths (potential wildcards)
        for route in &self.routes {
            if route.matches(request.uri().path()) {
                return if request.method() != route.method {
                    Err(HttpError::new(405, "Method not allowed".to_string()))
                } else {
                    Ok(Some(route))
                };
            }
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub struct Route {
    name: Option<&'static str>,
    method: Method,
    path: &'static str,
    segments: Vec<RouteSegment>,
    action: Box<dyn Action + 'static>,
}

impl Route {
    pub fn new<A: Action + 'static>(method: Method, path: &'static str, action: A) -> Route {
        // todo validate or normalize leading slash?
        Route {
            name: None,
            method,
            path,
            segments: process_segments(split_segments(path)),
            action: Box::new(action),
        }
    }

    pub fn path(&self) -> &'static str {
        self.path
    }

    pub fn action(&self) -> &Box<dyn Action + 'static> {
        &self.action
    }

    pub fn name(mut self, name: &'static str) -> Route {
        self.name = Some(name);
        self
    }

    pub fn get_name(&self) -> Option<&'static str> {
        self.name
    }

    pub fn constrain(mut self, parameter: &'static str, pattern: &'static str) -> Route {
        for segment in &mut self.segments {
            for variable in &mut segment.variables {
                if variable.handle == parameter {
                    variable.constrain(pattern);
                    break;
                }
            }
        }
        self
    }

    pub fn wildcard(mut self, parameter: &'static str, enable: bool) -> Route {
        for segment in &mut self.segments {
            for variable in &mut segment.variables {
                if variable.handle == parameter {
                    variable.wildcard(enable);
                    break;
                }
            }
        }
        self
    }

    pub fn matches(&self, path: &str) -> bool {
        let req_segs = split_segments(path).collect::<Vec<&str>>();
        let rou_segs = &self.segments;
        let mut is_match = true;
        let mut step = 0;

        // Loop over both request segments and route segments
        // and process each segment. todo check for wildcard variables
        loop {
            let req_seg = req_segs.iter().nth(step);
            let rou_seg = rou_segs.iter().nth(step);

            // Both ran out at same time, break out of loop and return the current value of matches
            if let None = req_seg
                && let None = rou_seg
            {
                return is_match;
            }

            // One ran out before the other. Segment counts do not match. Return false.
            if let Some(_) = req_seg
                && let None = rou_seg
            {
                return false;
            }

            // One ran out before the other. Segment counts do not match. Return false.
            if let None = req_seg
                && let Some(_) = rou_seg
            {
                return false;
            }

            if let Some(req_seg) = req_seg
                && let Some(rou_seg) = rou_seg
            {
                let has_variables = rou_seg.variables.len() > 0;

                if has_variables {
                    println!("after has_variables check");
                    for variable in &rou_seg.variables {
                        dbg!(&variable);
                        match variable.constraint {
                            Constraint::Default => {
                                is_match = true
                            }
                            Constraint::Wildcard(enabled) => {
                                if enabled {
                                    let start = variable.range.start;
                                    if req_seg[..start] == rou_seg.segment[..start] {
                                        // Return true out of matches function to mark as the resolved route
                                        return true;
                                    }
                                }
                            }
                            Constraint::Regex(_) => {
                                // todo
                            }
                        }
                    }
                } else {
                    is_match = *req_seg == rou_seg.segment;
                }

                if !is_match {
                    break;
                }
            }

            step += 1;
        }

        is_match
    }
}

#[derive(Debug)]
struct RouteSegment {
    segment: &'static str,
    variables: Vec<RouteVariable>,
}

impl RouteSegment {
    pub fn new(segment: &'static str) -> RouteSegment {
        let matches: Vec<RouteVariable> = REG
            .find_iter(segment)
            .map(|m| {
                RouteVariable {
                    handle: VAR_DELIMITER.replace_all(m.as_str(), ""),
                    range: m.range(),
                    constraint: Constraint::Default,
                }
            })
            .collect();

        RouteSegment {
            segment,
            variables: matches,
        }
    }
}

#[derive(Debug)]
struct RouteVariable {
    handle: Cow<'static, str>,
    range: Range<usize>,
    constraint: Constraint,
}

impl RouteVariable {
    fn wildcard(&mut self, enable: bool) -> &RouteVariable {
        self.constraint = Constraint::Wildcard(enable);
        self
    }

    fn constrain(&mut self, pattern: &'static str) -> &RouteVariable {
        self.constraint = Constraint::Regex(Regex::new(pattern).unwrap());
        self
    }
}

#[derive(Debug)]
enum Constraint {
    Default,
    Wildcard(bool),
    Regex(Regex),
}

/// Since we want to split the route definition path and the request
/// instance path the same way we will extract it into a helper fn
fn split_segments<'a>(path: &'a str) -> Split<'a, &'static str> {
    path.split("/")
}

fn process_segments(segments: Split<'static, &'static str>) -> Vec<RouteSegment> {
    let mut result = Vec::new();

    for segment in segments.into_iter() {
        result.push(RouteSegment::new(segment));
    }

    result
}

fn is_variable(segment: &str) -> bool {
    todo!()
}

fn is_model(segment: &str) -> bool {
    todo!()
}
