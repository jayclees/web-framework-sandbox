use crate::routing::action::Action;
use crate::routing::split_segments;
use crate::routing::tokenizer::{Constraint, SegmentTokenizer, Token};
use hyper::Method;
use regex::Regex;
use std::sync::LazyLock;

static DEFAULT_VAR_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(.*){1,}").unwrap());

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

    pub fn get_method(&self) -> &Method {
        &self.method
    }

    pub fn constrain(mut self, parameter: &'static str, pattern: &'static str) -> Route {
        let handle = format!("{{{parameter}}}");
        for segment in &mut self.segments {
            for token in &mut segment.tokens {
                if token.slice == handle {
                    token.constrain(pattern);
                    return self;
                }
            }
        }
        panic!("Parameter not found");
    }

    pub fn wildcard(mut self, parameter: &'static str, enable: bool) -> Route {
        let handle = format!("{{{parameter}}}");
        for segment in &mut self.segments {
            for token in &mut segment.tokens {
                if token.slice == handle {
                    if token.constraint == Constraint::Static {
                        panic!("Cannot set static token as wildcard.")
                    }

                    token.wildcard(enable);
                    return self;
                };
            }
        }
        panic!("Parameter not found");
    }

    pub fn matches(&self, path: &str) -> bool {
        let req_segs = split_segments(path);
        let rou_segs = &self.segments;

        return cmp(req_segs, rou_segs, 0);

        fn cmp(req_segs: Vec<&str>, rou_segs: &Vec<RouteSegment>, depth: usize) -> bool {
            let req_seg = req_segs.iter().nth(depth);
            let rou_seg = rou_segs.iter().nth(depth);

            if let None = req_seg
                && let None = rou_seg
            {
                // Recursive checks reached all the way to the end without failing, and both
                // ended at the same depth. This means we've found a match. Return true.
                return true;
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

            if reconcile_segs(req_seg.unwrap(), rou_seg.unwrap().tokens.clone(), depth) {
                return cmp(req_segs, rou_segs, depth + 1);
            }

            false
        }

        fn reconcile_segs(req_seg: &str, tokens: Vec<Token>, _depth: usize) -> bool {
            let mut cursor = 0;
            // if any of these checks fail, break out of loop and return false
            for token in tokens {
                let is_match = match token.constraint {
                    Constraint::Static => {
                        let slices_match = req_seg.len() >= token.range.end
                            && &req_seg[token.range.clone()] == token.slice;
                        if slices_match {
                            cursor = token.range.end;
                            true
                        } else {
                            false
                        }
                    }
                    Constraint::Default => {
                        let found = DEFAULT_VAR_PATTERN.find_at(req_seg, cursor);
                        if let None = found {
                            false
                        } else {
                            cursor = found.unwrap().range().end;
                            true
                        }
                    }
                    Constraint::Regex(regex) => {
                        let found = regex.find_at(req_seg, cursor);
                        if let None = found {
                            false
                        } else {
                            cursor = found.unwrap().range().end;
                            true
                        }
                    }
                    Constraint::Wildcard => {
                        // if cursor is at start range for token.range
                        if cursor == token.range.start {
                            // wildcard token starts at correct position in req_seg, return true for entire route
                            return true;
                        }
                        false
                    }
                };

                if !is_match {
                    return false;
                }
            }

            true
        }
    }
}

#[derive(Debug)]
struct RouteSegment {
    _segment: &'static str,
    tokens: Vec<Token>,
}

impl RouteSegment {
    pub fn new(segment: &'static str) -> RouteSegment {
        RouteSegment {
            _segment: segment,
            tokens: SegmentTokenizer::new(segment).tokenize(),
        }
    }
}

fn process_segments(segments: Vec<&'static str>) -> Vec<RouteSegment> {
    let mut result = Vec::new();

    for segment in segments.into_iter() {
        result.push(RouteSegment::new(segment));
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::error::HttpError;
    use crate::routing::action::{Action, Responsable};
    use crate::routing::router::Router;
    use async_trait::async_trait;
    use hyper::body::Incoming;
    use hyper::{Method, Request};
    use std::sync::LazyLock;

    static ROUTER: LazyLock<Router> = LazyLock::new(|| Router::new(register_routes));

    fn register_routes(router: &mut Router) {
        // Some of these routes are here to check that they are NOT
        // hit, so please don't remove any routes.
        router.get("/", GenericPage("Landing page"), None);

        router.get("/home", GenericPage("Home page"), None);
        router.get("/about", GenericPage("About us page"), None);

        router.get("/home/trending", GenericPage("Trending page"), None);
        router.get("/home/popular", GenericPage("Popular page"), None);

        router.get(
            "/home/settings/profile",
            GenericPage("Profile settings page"),
            None,
        );
        router.get(
            "/home/settings/preferences",
            GenericPage("Preferences settings page"),
            None,
        );

        // Variable testing
        router.get("/user/index", GenericPage("Show user index page"), None);
        router.get("/user/{user}", GenericPage("Show user page"), None);
        router.get(
            "/user/{user}/details",
            GenericPage("Show user details page"),
            None,
        );
        router.get(
            "/user/{user}/edit",
            GenericPage("Show user edit page"),
            None,
        );
        router.get(
            "/user/{user}/posts/featured",
            GenericPage("Show user posts page"),
            None,
        );

        // For constraint testing
    }

    #[test]
    fn resolve_landing_route() {
        let resolved = ROUTER.resolve_inner("/", &Method::GET).unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_eq!("/", resolved.unwrap().path)
    }

    #[test]
    fn resolve_route_one_level() {
        let resolved = ROUTER.resolve_inner("/about", &Method::GET).unwrap();

        if let None = resolved {
            assert!(false, "Route not resolved.");
        }

        assert_eq!("/about", resolved.unwrap().path);
    }

    #[test]
    fn resolve_route_two_levels_a() {
        let resolved = ROUTER
            .resolve_inner("/home/trending", &Method::GET)
            .unwrap();

        if let None = resolved {
            assert!(false, "Route not resolved.");
        }

        assert_eq!("/home/trending", resolved.unwrap().path);
    }

    #[test]
    fn resolve_route_two_levels_b() {
        let resolved = ROUTER.resolve_inner("/home/popular", &Method::GET).unwrap();

        if let None = resolved {
            assert!(false, "Route not resolved.");
        }

        assert_eq!("/home/popular", resolved.unwrap().path);
    }

    #[test]
    fn resolve_route_three_levels_a() {
        let resolved = ROUTER
            .resolve_inner("/home/settings/profile", &Method::GET)
            .unwrap();

        if let None = resolved {
            assert!(false, "Route not resolved.");
        }

        assert_eq!("/home/settings/profile", resolved.unwrap().path);
    }

    #[test]
    fn resolve_route_three_levels_b() {
        let resolved = ROUTER
            .resolve_inner("/home/settings/preferences", &Method::GET)
            .unwrap();

        if let None = resolved {
            assert!(false, "Route not resolved.");
        }

        assert_eq!("/home/settings/preferences", resolved.unwrap().path);
    }

    #[test]
    fn resolve_static_route_before_variable_route() {
        let resolved = ROUTER.resolve_inner("/user/index", &Method::GET).unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_ne!("/user/{user}", resolved.unwrap().path);
        assert_eq!("/user/index", resolved.unwrap().path);
    }

    #[test]
    fn resolve_variable_route() {
        let resolved = ROUTER.resolve_inner("/user/123", &Method::GET).unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_ne!("/user/index", resolved.unwrap().path);
        assert_eq!("/user/{user}", resolved.unwrap().path);
    }

    #[test]
    fn resolve_variable_route_depth_plus_one_a() {
        let resolved = ROUTER
            .resolve_inner("/user/123/details", &Method::GET)
            .unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_eq!("/user/{user}/details", resolved.unwrap().path)
    }

    #[test]
    fn resolve_variable_route_depth_plus_one_b() {
        let resolved = ROUTER
            .resolve_inner("/user/123/edit", &Method::GET)
            .unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_eq!("/user/{user}/edit", resolved.unwrap().path)
    }

    #[test]
    fn resolve_variable_route_depth_plus_two() {
        let resolved = ROUTER
            .resolve_inner("/user/123/posts/featured", &Method::GET)
            .unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_eq!("/user/{user}/posts/featured", resolved.unwrap().path)
    }

    #[derive(Debug)]
    struct GenericPage(&'static str);

    #[async_trait]
    impl Action for GenericPage {
        async fn handle(
            &self,
            _app: &App,
            _request: Request<Incoming>,
        ) -> Result<Box<dyn Responsable>, HttpError> {
            Ok(Box::new(self.0.to_string()))
        }
    }
}
