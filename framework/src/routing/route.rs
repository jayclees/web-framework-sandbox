use crate::routing::action::Action;
use crate::routing::split_segments;
use crate::routing::tokenizer::{Constraint, SegmentTokenizer, Token};
use hyper::Method;
use regex::{Match, Regex};
use std::sync::LazyLock;

static DEFAULT_VAR_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(.*){1,}").unwrap());

#[derive(Debug)]
pub struct Route {
    name: Option<String>,
    method: Method,
    path: String,
    segments: Vec<RouteSegment>,
    action: Box<dyn Action + 'static>,
}

impl Route {
    pub fn new<A: Action + 'static>(method: Method, path: String, action: A) -> Route {
        // todo validate or normalize leading slash?
        Route {
            name: None,
            method,
            path: path.clone(),
            segments: process_segments(split_segments(path)),
            action: Box::new(action),
        }
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn action(&self) -> &Box<dyn Action + 'static> {
        &self.action
    }

    pub fn name(mut self, name: String) -> Route {
        self.name = Some(name);
        self
    }

    pub fn get_name(&self) -> &Option<String> {
        &self.name
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
                    token.wildcard(enable);
                    return self;
                };
            }
        }
        panic!("Parameter not found");
    }

    pub fn matches(&self, path: &str) -> bool {
        let req_segs = split_segments(path.to_owned());
        let rou_segs = &self.segments;

        return cmp(req_segs, rou_segs, 0);

        fn cmp(req_segs: Vec<String>, rou_segs: &Vec<RouteSegment>, depth: usize) -> bool {
            let req_seg = req_segs.iter().nth(depth);
            let rou_seg = rou_segs.iter().nth(depth);

            // Recursive checks reached all the way to the end without failing, and both
            // ended at the same depth. This means we've found a match. Return true.
            if let None = req_seg
                && let None = rou_seg
            {
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
                    Constraint::Default => reconcile_regex(None, req_seg, &mut cursor),
                    Constraint::Regex(regex) => reconcile_regex(Some(regex), req_seg, &mut cursor),
                    Constraint::Wildcard => {
                        // If cursor matches start range for token.range
                        if cursor == token.range.start {
                            // Wildcard token starts at correct position in
                            // req_seg, return true for entire route
                            return true;
                        }
                        false
                    }
                };

                if !is_match {
                    return false;
                }

                // There is remaining unreconciled characters after all checks
                // todo test this
                // if cursor != req_seg.len() {
                //     return false;
                // }
            }

            true
        }

        fn reconcile_regex(regex: Option<Regex>, req_seg: &str, cursor: &mut usize) -> bool {
            // First check that the string slice being tested only contains
            // the characters allowed in the regex constraint.
            let is_match: bool;
            if let Some(regex) = &regex {
                is_match = regex.is_match_at(req_seg, *cursor);
                if req_seg == "123abc" {
                    dbg!(is_match, regex.as_str(), req_seg);
                }
            } else {
                is_match = DEFAULT_VAR_PATTERN.is_match_at(req_seg, *cursor);
            }
            if !is_match {
                return false;
            }

            let found: Option<Match>;
            if let Some(regex) = regex {
                found = regex.find_at(req_seg, *cursor);
            } else {
                found = DEFAULT_VAR_PATTERN.find_at(req_seg, *cursor);
            }

            if let None = found {
                false
            } else {
                *cursor = found.unwrap().range().end;
                true
            }
        }
    }
}

#[derive(Debug)]
struct RouteSegment {
    _segment: String,
    tokens: Vec<Token>,
}

impl RouteSegment {
    pub fn new(seg: String) -> RouteSegment {
        RouteSegment {
            _segment: seg.clone(),
            tokens: SegmentTokenizer::new(seg).tokenize(),
        }
    }
}

fn process_segments(segments: Vec<String>) -> Vec<RouteSegment> {
    segments
        .into_iter()
        .map(|seg| RouteSegment::new(seg))
        .collect()
    // let mut result = Vec::new();
    //
    // for segment in segments.into_iter() {
    //     result.push(RouteSegment::new(segment));
    // }
    //
    // result
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
        router.get("/".to_string(), GenericAction("Landing page"));

        router.get("/home".to_string(), GenericAction("Home page"));
        router.get("/about".to_string(), GenericAction("About us page"));

        router.get("/home/trending".to_string(), GenericAction("Trending page"));
        router.get("/home/popular".to_string(), GenericAction("Popular page"));

        router.get(
            "/home/settings/profile".to_string(),
            GenericAction("Profile settings page"),
        );
        router.get(
            "/home/settings/preferences".to_string(),
            GenericAction("Preferences settings page"),
        );

        // Variable testing
        router.get("/user/index".to_string(), GenericAction("Show user index page"));
        router.get("/user/{user}".to_string(), GenericAction("Show user page"));
        router.get(
            "/user/{user}/details".to_string(),
            GenericAction("Show user details page"),
        );
        router.get("/user/{user}/edit".to_string(), GenericAction("Show user edit page"));
        router.get(
            "/user/{user}/posts/featured".to_string(),
            GenericAction("Show user posts page"),
        );

        // For constraint testing
        router.getm(
            "/author/{name}".to_string(),
            GenericAction("Get author by name (alpha chars)"),
            |route| route.constrain("name", "[a-zA-Z]+"),
        );
        router.getm(
            "/author/{id}".to_string(),
            GenericAction("Get author by id (numeric chars)"),
            |route| route.constrain("id", "[0-9]+"),
        );
    }

    #[test]
    fn resolve_landing_route() {
        let resolved = ROUTER.resolve_inner("/", &Method::GET).unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_eq!("/", resolved.unwrap().path);
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
        assert_eq!("/user/{user}/details", resolved.unwrap().path);
    }

    #[test]
    fn resolve_variable_route_depth_plus_one_b() {
        let resolved = ROUTER
            .resolve_inner("/user/123/edit", &Method::GET)
            .unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_eq!("/user/{user}/edit", resolved.unwrap().path);
    }

    #[test]
    fn resolve_variable_route_depth_plus_two() {
        let resolved = ROUTER
            .resolve_inner("/user/123/posts/featured", &Method::GET)
            .unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_eq!("/user/{user}/posts/featured", resolved.unwrap().path);
    }

    #[test]
    fn resolve_variable_constraints_a() {
        let resolved = ROUTER
            .resolve_inner("/author/johndoe", &Method::GET)
            .unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_ne!("/author/{id}", resolved.unwrap().path);
        assert_eq!("/author/{name}", resolved.unwrap().path);
    }

    #[test]
    fn resolve_variable_constraints_b() {
        let resolved = ROUTER.resolve_inner("/author/123", &Method::GET).unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_ne!("/author/{user}", resolved.unwrap().path);
        assert_eq!("/author/{id}", resolved.unwrap().path);
    }

    #[test]
    fn resolve_variable_constraints_c() {
        // Test mixed alpha/num. Since we have two routes that each test
        // exclusively for alphabetical chars or numeric chars, a
        // mixed route parameter should resolve to neither.
        let resolved = ROUTER
            .resolve_inner("/author/abc123", &Method::GET)
            .unwrap();
        if let Some(route) = resolved {
            assert!(
                false,
                "A route was resolved when none should have., \"{}\"",
                route.path
            );
        }
    }

    #[test]
    fn resolve_variable_constraints_d() {
        // Test mixed alpha/num. Since we have two routes that each test
        // exclusively for alphabetical chars or numeric chars, a
        // mixed route parameter should resolve to neither.
        let resolved = ROUTER
            .resolve_inner("/author/123abc", &Method::GET)
            .unwrap();
        if let Some(route) = resolved {
            assert!(
                false,
                "A route was resolved when none should have., \"{}\"",
                route.path
            );
        }
    }

    #[derive(Debug)]
    struct GenericAction(&'static str);

    #[async_trait]
    impl Action for GenericAction {
        async fn handle(
            &self,
            _app: &App,
            _request: Request<Incoming>,
        ) -> Result<Box<dyn Responsable>, HttpError> {
            Ok(Box::new(self.0.to_string()))
        }
    }
}
