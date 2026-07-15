use crate::action::Action;
use crate::routing::split_segments;
use crate::routing::tokenizer::{Constraint, SegmentTokenizer, Token};
use hyper::Method;
use regex::Regex;
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

    pub fn name(mut self, name: &str) -> Route {
        self.name = Some(name.to_owned());
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

        SegmentReconciliator::new(req_segs, self).cmp()
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
}

struct SegmentReconciliator<'a> {
    _route: &'a Route,
    req_segs: Vec<String>,
    rou_segs: &'a Vec<RouteSegment>,
    depth: usize,
}

impl<'a> SegmentReconciliator<'a> {
    fn new(req_segs: Vec<String>, route: &'a Route) -> SegmentReconciliator<'a> {
        // dbg!(&route.path); // Uncomment this to see which route you are in when debugging
        SegmentReconciliator {
            _route: route,
            req_segs,
            rou_segs: &route.segments,
            depth: 0,
        }
    }

    fn cmp(&mut self) -> bool {
        let req_seg = self.req_segs.iter().nth(self.depth);
        let rou_seg = self.rou_segs.iter().nth(self.depth);

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

        if self.reconcile_segs(req_seg.unwrap(), &rou_seg.unwrap()) {
            self.depth += 1;
            return self.cmp();
        }

        false
    }

    fn reconcile_segs(&self, req_seg: &str, rou_seg: &RouteSegment) -> bool {
        let mut cursor = 0;
        let mut i = 0;
        let tok_len = rou_seg.tokens.len();

        // If any of these checks fail, break out of loop and return false.
        for token in &rou_seg.tokens {
            let is_match = match &token.constraint {
                Constraint::Static => {
                    let start = cursor;
                    let end = cursor + token.slice.len();
                    let slices_match = req_seg.len() >= end && &req_seg[start..end] == token.slice;
                    if slices_match {
                        cursor += token.slice.len();
                        true
                    } else {
                        false
                    }
                }
                Constraint::Default => {
                    if let Some(found) = DEFAULT_VAR_PATTERN.find_at(req_seg, cursor) {
                        cursor = found.range().end;
                        // todo copy from Constraint::Regex arm
                        true
                    } else {
                        false
                    }
                }
                Constraint::Regex(regex) => {
                    let is_match = if i == 0 && tok_len == 1 {
                        // If there is only one token, match full req_seg.
                        if let Some(found) = regex.find(req_seg) {
                            if found.as_str() == req_seg {
                                // After the loop ends the cursor being at
                                // end means everything properly matched.
                                cursor = found.as_str().len();
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else if i < tok_len - 1 {
                        // If token has proceeding tokens, find match. If none found,
                        // return false, if found, save cursor position and continue.
                        if let Some(found) = regex.find_at(req_seg, cursor) {
                            if found.is_empty() {
                                false
                            } else if found.range().end == req_seg.len() {
                                // The token matched until the end, but there should have
                                // been at least one more token we needed to match.
                                // todo:
                                // Perhaps panic? If not here, during boot phase we can ensure that if there are
                                // variable tokens that have other tokens following it, that the user must
                                // add a regex constraint, other the default will match all characters.
                                false
                            } else {
                                // We will assume for now it's true. We will know by end of
                                // this segment's reconciliation if it's a match or not.
                                cursor = found.range().end;
                                true
                            }
                        } else {
                            false
                        }
                    } else if i == tok_len - 1 {
                        // If token is the last one, match exactly. If the
                        // match is exact, continue, else return false.
                        if let Some(found) = regex.find_at(req_seg, cursor) {
                            found.as_str() == req_seg
                        } else {
                            false
                        }
                    } else {
                        unimplemented!("If this else block is hit, we missed something.");
                    };

                    if !is_match {
                        return false;
                    }

                    true
                }
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

            i += 1;
        }

        // If cursor is less than req seg len, match was invalid
        cursor == req_seg.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::action::{Action, Responsable};
    use crate::app::App;
    use crate::error::HttpError;
    use crate::routing::router::Router;
    use async_trait::async_trait;
    use hyper::body::Incoming;
    use hyper::{Method, Request};
    use std::sync::LazyLock;

    static ROUTER: LazyLock<Router> = LazyLock::new(|| Router::new(register_routes));

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

        // /{id:numeric}
        // /post.{id:numeric}
        // /post.{id:numeric}.{slug:alpha_dash}
        // /post.{id:numeric}.{slug:alpha_dash}.{year}
        if let Some(route) = resolved {
            assert!(
                false,
                "A route was resolved when none should have., \"{}\"",
                route.path
            );
        }
    }

    #[test]
    fn reconcile_multi_token_segments() {
        let resolved = ROUTER
            .resolve_inner("/post/johndoe.123456789.how-to-do-thing", &Method::GET)
            .unwrap();
        if let None = resolved {
            assert!(false, "Route not resolved.");
        }
        assert_eq!(
            "/post/{author}.{post_id}.{post_title}",
            resolved.unwrap().path
        );
    }

    fn register_routes(router: &mut Router) {
        // Some of these routes are here to check that they are NOT
        // hit, so please don't remove any routes.
        router.get("/", Generic);

        router.get("/home", Generic);
        router.get("/about", Generic);

        router.get("/home/trending", Generic);
        router.get("/home/popular", Generic);

        router.get("/home/settings/profile", Generic);
        router.get("/home/settings/preferences", Generic);

        // Variable testing
        router.get("/user/index", Generic);
        router.get("/user/{user}", Generic);
        router.get("/user/{user}/details", Generic);
        router.get("/user/{user}/edit", Generic);
        router.get("/user/{user}/posts/featured", Generic);

        // For constraint testing
        router.getm("/author/{name}", Generic, |route| {
            route.constrain("name", "[a-zA-Z]+")
        });
        router.getm("/author/{id}", Generic, |route| {
            route.constrain("id", "[0-9]+")
        });

        // multi-token segments
        router.getm("/post/{author}.", Generic, |route| {
            route.constrain("author", "[a-zA-Z]+")
            // .constrain("post_id", "[0-9]+")
        });
        router.getm("/post/{author}.{post_id}.{post_title}", Generic, |route| {
            route
                .constrain("author", "[a-zA-Z]+")
                .constrain("post_id", "[0-9]+")
        });
    }

    #[derive(Debug)]
    struct Generic;

    #[async_trait]
    impl Action for Generic {
        async fn handle(
            &self,
            _app: &App,
            _request: Request<Incoming>,
        ) -> Result<Box<dyn Responsable>, HttpError> {
            Ok(Box::new("hello".to_string()))
        }
    }
}
