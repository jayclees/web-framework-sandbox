use crate::action::Action;
use regex::Regex;
use std::sync::LazyLock;

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

    pub fn resolve(&self, path: &str) -> Option<&Route> {
        for route in &self.routes {
            if route.path() == path {
                return Some(route);
            }
        }
        None
    }
}

enum PathPart {
    String(&'static str),
    Variable(&'static str, Vec<&'static str>),
    ModelId(&'static str),
    // regular string path
    // model string name
}

type ActionType = Box<dyn Action + Send + Sync>;

pub struct Route {
    method: &'static str,
    path: &'static str,
    path_parts: Vec<PathPart>,
    action: ActionType,
}

impl Route {
    pub fn new(method: &'static str, path: &'static str, action: ActionType) -> Route {
        Route {
            method,
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn get(path: &'static str, action: ActionType) -> Route {
        Route {
            method: "get",
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn post(path: &'static str, action: ActionType) -> Route {
        Route {
            method: "post",
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn patch(path: &'static str, action: ActionType) -> Route {
        Route {
            method: "patch",
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn put(path: &'static str, action: ActionType) -> Route {
        Route {
            method: "put",
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn delete(path: &'static str, action: ActionType) -> Route {
        Route {
            method: "delete",
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn head(path: &'static str, action: ActionType) -> Route {
        Route {
            method: "head",
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn connect(path: &'static str, action: ActionType) -> Route {
        Route {
            method: "connect",
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn options(path: &'static str, action: ActionType) -> Route {
        Route {
            method: "options",
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn trace(path: &'static str, action: ActionType) -> Route {
        Route {
            method: "trace",
            path,
            path_parts: Self::split_parts(path),
            action,
        }
    }

    pub fn path(&self) -> &'static str {
        self.path
    }

    pub fn action(&self) -> &ActionType {
        &self.action
    }

    fn split_parts(path: &'static str) -> Vec<PathPart> {
        let mut result = Vec::new();
        let parts = path.split("/");

        for part in parts.into_iter() {
            result.push(Self::part_type(part));
        }

        result
    }

    fn part_type(part: &'static str) -> PathPart {
        // todo: avoid recompiling regex every time
        static REG: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\{[^_][a-zA-Z0-9_]*[a-zA-Z0-9]}").unwrap());
        let matches: Vec<&str> = REG.find_iter(part).map(|m| m.as_str()).collect();

        if matches.len() == 0 {
            return PathPart::String(part);
        }

        PathPart::Variable(part, matches)
    }

    fn is_variable(part: &str) -> bool {
        todo!()
    }

    fn is_model(part: &str) -> bool {
        todo!()
    }
}
