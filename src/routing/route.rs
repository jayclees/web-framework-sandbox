use std::borrow::Cow;
use std::ops::Range;
use std::str::Split;
use std::sync::LazyLock;
use hyper::Method;
use regex::Regex;
use crate::action::Action;
use crate::routing::{split_segments, SegmentTokenizer, Token, TokenType};

static REG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^_][a-zA-Z0-9_]*[a-zA-Z0-9]}").unwrap());

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
                // let has_variables = rou_seg.variables.len() > 0;
                //
                // if has_variables {
                //     for variable in &rou_seg.variables {
                //         let start = variable.range.start;
                //         let start_is_match = req_seg[..start] == rou_seg.segment[..start];
                //
                //         if ! start_is_match {
                //             is_match = false;
                //             break;
                //         }
                //
                //         is_match = match &variable.constraint {
                //             Constraint::Default => true,
                //             Constraint::Wildcard(enabled) => {
                //                 if *enabled {
                //                     // Return true out of matches function to mark as the resolved route
                //                     return true;
                //                 }
                //                 false
                //             }
                //             Constraint::Regex(pattern) => {
                //                 false
                //             },
                //         }
                //     }
                // } else {
                //     is_match = *req_seg == rou_seg.segment;
                // }

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

fn process_segments(segments: Split<'static, &'static str>) -> Vec<RouteSegment> {
    let mut result = Vec::new();

    for segment in segments.into_iter() {
        result.push(RouteSegment::new(segment));
    }

    result
}
