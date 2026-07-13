use std::ops::{Index, Range};
use std::str::Split;

mod route;
pub mod router;

/// Since we want to split the route definition path and the request
/// instance path the same way we will extract it into a helper fn
fn split_segments<'a>(path: &'a str) -> Split<'a, &'static str> {
    path.split("/")
}

#[derive(Debug)]
struct SegmentTokenizer {
    state: State,
    segment: &'static str,
    state_start: usize,
}

impl SegmentTokenizer {
    fn new(segment: &'static str) -> SegmentTokenizer {
        SegmentTokenizer {
            state: State::Default,
            segment,
            state_start: 0,
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];

        for (i, char) in self.segment.chars().enumerate() {
            match char {
                '{' => {
                    // If this is the last character, break loop and
                    // consider this curly brace a static character
                    if i == self.segment.len() - 1 {
                        break;
                    }

                    if i > self.state_start {
                        let range = self.state_start..i;
                        tokens.push(Token::Static(range.clone(), &self.segment[range]));
                    }

                    self.change_state(State::InCurly, i);
                }
                '}' => {
                    match self.state {
                        State::InCurly => {
                            if self.state_start + 1 == i {
                                // Curly braces closed immediately after, treat them as
                                // static. If previous token is Token::Static(..),
                                // push last two chars "{}" onto previous token.
                                self.state = State::Default;
                                if let Some(mut token) = tokens.last_mut()
                                    && let Token::Static(range, slice) = &mut token
                                {
                                    // overwrite last token to include the last 2 chars "{}"
                                    let new_range = range.start..range.end + 2;
                                    *token = Token::Static(new_range.clone(), &self.segment[new_range]);
                                    self.change_state(State::Default, i + 1);
                                }
                            } else {
                                let range = self.state_start..i + 1;
                                tokens.push(Token::Variable(range.clone(), &self.segment[range]));
                                self.change_state(State::Default, i + 1);
                            }
                        }
                        State::Default => {
                            // Do nothing?
                        }
                    }
                }
                _ => {
                    // Do nothing?
                }
            }
        }

        if self.state_start != self.segment.len() {
            let range = self.state_start..self.segment.len();
            tokens.push(Token::Static(range.clone(), &self.segment[range]));
        }

        tokens
    }

    pub fn change_state(&mut self, state: State, index: usize) {
        self.state = state;
        self.state_start = index;
    }
}

#[derive(Debug, PartialEq)]
enum State {
    Default,
    InCurly,
}

#[derive(Debug, PartialEq)]
enum Token {
    Static(Range<usize>, &'static str),
    Variable(Range<usize>, &'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let segment = "test";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::Static(0..segment.len(), "test")];
        assert_eq!(expect, tokenizer.tokenize());

        //

        let segment = "{var}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::Variable(0..segment.len(), "{var}")];
        assert_eq!(expect, tokenizer.tokenize());

        //

        let segment = "{var_1}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::Variable(0..segment.len(), "{var_1}")];
        assert_eq!(expect, tokenizer.tokenize());

        //

        let segment = "test-{var1}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::Static(0..5, "test-"),
            Token::Variable(5..segment.len(), "{var1}"),
        ];
        assert_eq!(expect, tokenizer.tokenize());

        //

        let segment = "{var}-end";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::Variable(0..5, "{var}"),
            Token::Static(5..segment.len(), "-end"),
        ];
        assert_eq!(expect, tokenizer.tokenize());

        //

        let segment = "test-{var1}-t2";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::Static(0..5, "test-"),
            Token::Variable(5..11, "{var1}"),
            Token::Static(11..segment.len(), "-t2"),
        ];
        assert_eq!(expect, tokenizer.tokenize());

        //

        let segment = "test-{var1}t2{var_2}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::Static(0..5, "test-"),
            Token::Variable(5..11, "{var1}"),
            Token::Static(11..13, "t2"),
            Token::Variable(13..segment.len(), "{var_2}"),
        ];
        assert_eq!(expect, tokenizer.tokenize());

        //

        let segment = "test-{var1}t2{var_2}-end";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::Static(0..5, "test-"),
            Token::Variable(5..11, "{var1}"),
            Token::Static(11..13, "t2"),
            Token::Variable(13..20, "{var_2}"),
            Token::Static(20..segment.len(), "-end"),
        ];
        assert_eq!(expect, tokenizer.tokenize());

        //

        // Treat open brace as static if end of str
        let segment = "test{";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::Static(0..5, "test{")];
        assert_eq!(expect, tokenizer.tokenize());

        //

        // Treat empty curly braces as static chars
        let segment = "test{}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::Static(0..6, "test{}")];
        assert_eq!(expect, tokenizer.tokenize());

        //

        // Treat close brace as static if end of str
        let segment = "test}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::Static(0..5, "test}")];
        assert_eq!(expect, tokenizer.tokenize());

        //

        let segment = "{}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::Static(0..2, "{}")];
        assert_eq!(expect, tokenizer.tokenize());
    }
}
