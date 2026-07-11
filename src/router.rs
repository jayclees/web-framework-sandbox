use crate::action::Action;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::str::Split;
use std::sync::LazyLock;

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

    pub fn resolve(&self, path: &str) -> Option<&Route> {
        let req_segments: Vec<&str> = split_segments(path).collect();

        // todo: handle constrained route parameter paths (potential wildcards)

        // Loop over routes
        for route in &self.routes {
            if route.matches(path) {
                return Some(route);
            }

            // // let mut route_segments = route.segments.iter();
            // let mut matches = true;
            //
            // for (i, segment) in route.segments.iter().enumerate() {
            //     // matches = req_segments.nth(i) != segment
            // }
            //
            // // Loop over request segments and check the route segment at the corresponding depth.
            // // If any check fails, set matches to false, break out of loop
            // // for req_segment in &req_segments {
            // //     let route_segment = route_segments.nth(i);
            // //
            // //     if let Some(segment) = route_segment {
            // //         matches = match segment {
            // //             RouteSegment::String(string) => string == req_segment,
            // //             RouteSegment::Variable { handle: _, matches: _ } => true, // todo add check for regex constraint match
            // //             RouteSegment::ModelId(_) => todo!(),
            // //         };
            // //     } else {
            // //         // route segments had less segments than
            // //         matches = false;
            // //     }
            // //
            // //     if !matches {
            // //         break;
            // //     }
            // //
            // //     i += 1;
            // // }
            //
            // if matches {
            //     return Some(route);
            // }
        }

        None
    }
}

type ActionType = Box<dyn Action + Send + Sync>;

#[derive(Debug)]
pub struct Route {
    // todo implement route names
    name: Option<&'static str>,
    // todo use http::Method(Inner) enum
    method: &'static str,
    path: &'static str,
    segments: Vec<RouteSegment<'static>>,
    action: ActionType,
    filter: Option<()>,
}

impl Route {
    pub fn new(method: &'static str, path: &'static str, action: ActionType) -> Route {
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
        Self::new("get", path, action)
    }

    pub fn post(path: &'static str, action: ActionType) -> Route {
        Self::new("post", path, action)
    }

    pub fn patch(path: &'static str, action: ActionType) -> Route {
        Self::new("patch", path, action)
    }

    pub fn put(path: &'static str, action: ActionType) -> Route {
        Self::new("put", path, action)
    }

    pub fn delete(path: &'static str, action: ActionType) -> Route {
        Self::new("delete", path, action)
    }

    pub fn head(path: &'static str, action: ActionType) -> Route {
        Self::new("head", path, action)
    }

    pub fn connect(path: &'static str, action: ActionType) -> Route {
        Self::new("connect", path, action)
    }

    pub fn options(path: &'static str, action: ActionType) -> Route {
        Self::new("options", path, action)
    }

    pub fn trace(path: &'static str, action: ActionType) -> Route {
        Self::new("trace", path, action)
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
        let mut step = 0;

        // Loop over both request segments and route segments
        // and process each segment. todo check for wildcard variables
        loop {
            if let Some(req_seg) = req_segs.iter().nth(step)
                && let Some(rou_seg) = rou_segs.iter().nth(step)
            {
                return match rou_seg {
                    RouteSegment::String(rou_seg) => rou_seg == req_seg,
                    RouteSegment::Variable {
                        handle: _,
                        matches: _,
                    } => {
                        // check if variable has regex constraint
                        false
                    }
                    RouteSegment::ModelId(_) => {
                        false
                    }
                };
            } else {
                return false;
            }

            match rou_segs.iter().nth(step) {
                None => {}
                Some(_) => {}
            }

            step += 1;
        }

        // if has variables,

        false
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
