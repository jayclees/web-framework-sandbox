use crate::action::Action;

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

    pub fn path(&self) -> &'static str {
        self.path
    }

    pub fn action(&self) -> &ActionType {
        &self.action
    }
}
