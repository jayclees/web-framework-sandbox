use crate::action::Action;
use regex::Regex;
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
        let req_parts: Vec<&str> = split_parts(path).collect();

        // todo: handle constrained route parameter paths (potential wildcards)

        // Loop over routes
        for route in &self.routes {
            let mut route_parts = route.path_parts.iter();
            let mut matches = true;
            let mut i = 0;

            // Loop over request part segments and check the route part at the corresponding depth.
            // If any check fails, set matches to false, break out of loop
            for req_part in &req_parts {
                let route_part = route_parts.nth(i);

                if let Some(part) = route_part {
                    matches = match part {
                        PathPart::String(string) => string == req_part,
                        PathPart::Variable(_, _) => true, // todo add check for regex constraint match
                        PathPart::ModelId(_) => todo!(),
                    };
                } else {
                    matches = false;
                }

                if !matches {
                    break;
                }

                i = i + 1;
            }

            if matches {
                return Some(route);
            }
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
    path_parts: Vec<PathPart<'static>>,
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
            path_parts: process_parts(split_parts(path)),
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

    pub fn constrain(&self, parameter: &str, pattern: &str) -> &Self {
        todo!("implement constraints for route parameter");
        self
    }
}

#[derive(Debug)]
enum PathPart<'a> {
    String(&'a str),
    Variable(&'a str, Vec<&'a str>),
    ModelId(&'a str),
    // regular string path
    // model string name
}

/// Since we want to split the route definition path and the request
/// instance path the same way we will extract it into a helper fn
fn split_parts<'a>(path: &'a str) -> Split<'a, &'static str> {
    path.split("/")
}

fn process_parts<'a>(parts: Split<'a, &'static str>) -> Vec<PathPart<'a>> {
    let mut result = Vec::new();

    for part in parts.into_iter() {
        result.push(part_type(part));
    }

    result
}

fn part_type(part: &str) -> PathPart {
    let matches: Vec<&str> = REG.find_iter(part).map(|m| m.as_str()).collect();

    if matches.len() == 0 {
        return PathPart::String(part);
    }

    // E.g. example.com/user/{user}
    //                       ^^^^^^
    PathPart::Variable(part, matches)
}

fn is_variable(part: &str) -> bool {
    todo!()
}

fn is_model(part: &str) -> bool {
    todo!()
}
