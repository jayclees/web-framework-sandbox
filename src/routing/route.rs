use crate::action::Action;
use crate::routing::split_segments;
use crate::routing::tokenizer::{SegmentTokenizer, Token, TokenType};
use hyper::Method;
use regex::Regex;
use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

static REG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^_][a-zA-Z0-9_]*[a-zA-Z0-9]}").unwrap());

static DEFAULT_VAR_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r".*").unwrap());

static VAR_DELIMITER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\A\{)|(})\z").unwrap());

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
        for segment in &mut self.segments {
            for token in &mut segment.tokens {
                if token.token_type == TokenType::Variable {
                    // token.constrain(pattern);
                    break;
                }
            }
        }
        self
    }

    pub fn wildcard(mut self, parameter: &'static str, enable: bool) -> Route {
        for segment in &mut self.segments {
            for token in &mut segment.tokens {
                if token.token_type == TokenType::Variable {
                    // token.wildcard(enable);
                    break;
                };
            }
        }
        self
    }

    pub fn matches(&self, path: &str) -> bool {
        let req_segs = split_segments(path);
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
                //
                let mut matches = vec![];
                let mut cursor: usize = 0;
                dbg!(req_seg);
                dbg!(&rou_seg.tokens);

                // /post/{author}.{post_id}.{post_slug}
                // /post/jake.123.how-to-do-thing

                // next pattern is alphabetic chars
                // ending
                // found author pattern "jake" cursor index is now at 4

                // next token_type is static Range(8..9).
                // 4 - 8 = -4 range.start + -4 and range.end + -4 = Range(4..5)
                // does &req_seg[4..5] == "."?
                // yes

                // next token_type is var, pattern is numbers
                // cursor index is at 5, so find regex match in & &req_seg[5..]
                //

                // first token:
                for token in &rou_seg.tokens {
                    match token.token_type {
                        TokenType::Static => {
                            if &req_seg.len() < &token.range.end {
                                is_match = false;
                                break;
                            }

                            if &req_seg[token.range.clone()] != token.slice {
                                is_match = false;
                                break;
                            }
                        }
                        TokenType::Variable => {
                            if let Some(found) = DEFAULT_VAR_PATTERN.find(req_seg) {
                                matches.push(found);
                            } else {
                                //
                            }
                        }
                    }
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
    tokens: Vec<Token>,
}

impl RouteSegment {
    pub fn new(segment: &'static str) -> RouteSegment {
        RouteSegment {
            segment,
            tokens: SegmentTokenizer::new(segment).tokenize(),
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

fn process_segments(segments: Vec<&'static str>) -> Vec<RouteSegment> {
    let mut result = Vec::new();

    for segment in segments.into_iter() {
        result.push(RouteSegment::new(segment));
    }

    result
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

    #[test]
    fn test_landing() {
        let resolved = ROUTER.resolve_inner("/", &Method::GET).unwrap().unwrap();
        assert_eq!(resolved.path, "/")
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
            None,
        );
        router.get("/static{var}", GenericPage("var test 1"), None);
        router.get("/static{var}end", GenericPage("var test 2"), None);
        router.get(
            "/{varstart}-mid-static-{varend}",
            GenericPage("var test 3"),
            None,
        );
        router.get("/app/{wildcard}", GenericPage("app wildcard test"), None);
        router.get("/{wildcard}", GenericPage("root wildcard test"), None);
        router.get(
            "/inaccessible-due-to-wildcard",
            GenericPage("can't reach"),
            None,
        );
    }
}
