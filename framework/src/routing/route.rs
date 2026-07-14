use crate::routing::action::Action;
use crate::routing::split_segments;
use crate::routing::tokenizer::{Constraint, SegmentTokenizer, Token, TokenType};
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
                if token.token_type == TokenType::Variable && token.slice == handle {
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
                if token.token_type == TokenType::Variable && token.slice == handle {
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

        // request path: /home/settings/preferences
        // Route list:
        // /
        // /home
        // /home/app
        // /home/posts
        // /home/trending
        // /home/settings
        // /home/settings/profile
        // /home/settings/preferences
        // /home/settings/security

        return cmp(req_segs, rou_segs, 0);

        fn cmp(req_segs: Vec<&str>, rou_segs: &Vec<RouteSegment>, depth: usize) -> bool {
            let req_seg = req_segs.iter().nth(depth);
            let rou_seg = rou_segs.iter().nth(depth);

            if let None = req_seg
                && let None = rou_seg
            {
                return false;
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

            if reconcile_segs(req_seg.unwrap(), rou_seg.unwrap().tokens.clone()) {
                return cmp(req_segs, rou_segs, depth + 1);
            }

            false
        }

        fn reconcile_segs(req_seg: &str, tokens: Vec<Token>) -> bool {
            let mut cursor = 0;
            // if any of these checks fail, break out of loop and return false
            for token in tokens {
                let is_match = match token.constraint {
                    Constraint::Static => {
                        let slices_match = req_seg.len() >= token.range.end
                            && &req_seg[token.range.clone()] == token.slice;
                        if slices_match {
                            cursor = token.range.end
                        }
                        true
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

// #[derive(Debug)]
// struct RouteVariable {
//     handle: Cow<'static, str>,
//     range: Range<usize>,
//     // constraint: Constraint,
// }

// impl RouteVariable {
//     fn wildcard(&mut self, enable: bool) -> &RouteVariable {
//         self.constraint = Constraint::Wildcard(enable);
//         self
//     }
//
//     fn constrain(&mut self, pattern: &'static str) -> &RouteVariable {
//         self.constraint = Constraint::Regex(Regex::new(pattern).unwrap());
//         self
//     }
// }

// #[derive(Debug)]
// enum Constraint {
//     Default,
//     Wildcard(bool),
//     Regex(Regex),
// }

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
        router.get("/", GenericPage("landing page"), None);
        router.get("/home", GenericPage("home page"), None);
        router.get("/register", GenericPage("register page"), None);
        router.post("/register", GenericPage("created account"), None);
        router.get("/about-us", GenericPage("about page"), None);
        router.get("/deeply/nested", GenericPage("deeply nested"), None);
        router.get(
            "/deeply/nested/route",
            GenericPage("deeply nested route"),
            None,
        );
        router.get(
            "/deeply/othernested",
            GenericPage("deeply other nested"),
            None,
        );
        router.get(
            "/deeply/othernested/route",
            GenericPage("deeply other nested route"),
            None,
        );
        router.get("/user", GenericPage("user index"), None);
        router.get("/user/{user}", GenericPage("user show"), None);
        router.get("/user/{user}/edit", GenericPage("user edit"), None);
        router.get("/user/{user}/post", GenericPage("user post index"), None);
        router.get(
            "/user/{user}/post/{author}.{post_id}.{post_slug}",
            GenericPage("blog article"),
            Some(|route| {
                route
                    .constrain("author", "[a-zA-Z]+")
                    .constrain("post_id", "[0-9]+")
            }),
        );
        router.get("/static{var}", GenericPage("var test 1"), None);
        router.get("/static{var}end", GenericPage("var test 2"), None);
        router.get(
            "/{varstart}-mid-static-{varend}",
            GenericPage("var test 3"),
            None,
        );
        router.get("/app/{wildcard}", GenericPage("app wildcard test"), None);
        router.get(
            "/{wildcard}",
            GenericPage("root wildcard test"),
            Some(|route| route.wildcard("wildcard", true)),
        );
        router.get(
            "/inaccessible-due-to-wildcard",
            GenericPage("can't reach"),
            None,
        );
    }

    #[test]
    fn test_landing() {
        let resolved = ROUTER.resolve_inner("/", &Method::GET).unwrap().unwrap();
        assert_eq!("/", resolved.path)
    }

    #[test]
    fn test_about() {
        let resolved = ROUTER.resolve_inner("/about-us", &Method::GET).unwrap();

        if let None = resolved {
            assert!(false, "Route not resolved.");
        }

        assert_eq!("/about-us", resolved.unwrap().path);
    }

    #[test]
    fn test_deeply_nested_route() {
        let resolved = ROUTER
            .resolve_inner("/deeply/nested/route", &Method::GET)
            .unwrap()
            .unwrap();
        assert_eq!("/deeply/nested/route", resolved.path)
    }

    #[test]
    fn test_deeply_othernested_route() {
        let resolved = ROUTER
            .resolve_inner("/deeply/othernested/route", &Method::GET)
            .unwrap()
            .unwrap();
        assert_eq!("/deeply/othernested/route", resolved.path)
    }

    #[test]
    fn test_user_variable() {
        let resolved = ROUTER
            .resolve_inner("/user/1", &Method::GET)
            .unwrap()
            .unwrap();
        assert_eq!("/user/{user}", resolved.path)
    }

    #[test]
    fn test_multiple_token_segments() {
        let resolved = ROUTER
            .resolve_inner("/user/123/post/johndoe.456.how-to-do-thing", &Method::GET)
            .unwrap()
            .unwrap();
        assert_eq!(
            "/user/{user}/post/{author}.{post_id}.{post_slug}",
            resolved.path
        )
    }

    #[test]
    fn test_app_wildcard() {
        let resolved = ROUTER
            .resolve_inner("/app/abc/123/foobar/hello-world", &Method::GET)
            .unwrap()
            .unwrap();
        assert_eq!("/app/{wildcard}", resolved.path)
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
