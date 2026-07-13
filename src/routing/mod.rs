use regex::Regex;
use std::ops::Range;
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
                        tokens.push(Token::new_stat(self.state_start..i, &self.segment));
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
                                    && token.token_type == TokenType::Static
                                {
                                    // overwrite last token to include the last 2 chars "{}"
                                    token.range.end += 2;
                                    token.slice = &self.segment[token.range.clone()];
                                    self.change_state(State::Default, i + 1);
                                }
                            } else {
                                tokens.push(Token::new_var(self.state_start..i + 1, self.segment));
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
            tokens.push(Token::new_stat(range.clone(), self.segment));
        }

        tokens
    }

    pub fn change_state(&mut self, state: State, index: usize) {
        self.state = state;
        self.state_start = index;
    }
}

#[derive(Debug)]
enum State {
    Default,
    InCurly,
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    range: Range<usize>,
    slice: &'static str,
    constraint: Constraint,
    // Static(Range<usize>, &'static str),
    // Variable(Range<usize>, &'static str),
}

impl Token {
    fn new_stat(range: Range<usize>, segment: &'static str) -> Token {
        let clone = range.clone();
        Token {
            token_type: TokenType::Static,
            range: clone,
            slice: &segment[range],
            constraint: Constraint::Default,
        }
    }

    fn new_var(range: Range<usize>, segment: &'static str) -> Token {
        let clone = range.clone();
        Token {
            token_type: TokenType::Variable,
            range: clone,
            slice: &segment[range],
            constraint: Constraint::Default,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Static,
    Variable,
}

#[derive(Debug)]
enum Constraint {
    Default,
    Wildcard,
    Regex(Regex),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{Display, Formatter};

    impl Display for Token {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            // token_type
            // range
            // slice
            write!(
                f,
                "Token {{ token_type: {}, range: {}, slice: \"{}\", constraint: {} }}",
                self.token_type,
                format!("Range {{ {}..{} }}", self.range.start, self.range.end),
                self.slice,
                self.constraint,
            )
        }
    }
    impl Display for TokenType {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                TokenType::Static => write!(f, "{}", "TokenType::Static"),
                TokenType::Variable => write!(f, "{}", "TokenType::Variable"),
            }
        }
    }

    impl Display for Constraint {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Constraint::Default => write!(f, "{}", "Constraint::Default"),
                Constraint::Wildcard => write!(f, "{}", "Constraint::Wildcard"),
                Constraint::Regex(re) => {
                    write!(f, "{}", format!("Constraint::Regex(\"{}\")", re.as_str()))
                }
            }
        }
    }

    impl Clone for Token {
        fn clone(&self) -> Token {
            Token {
                token_type: self.token_type.clone(),
                range: self.range.clone(),
                slice: self.slice.clone(),
                constraint: Constraint::Default.clone(),
            }
        }
    }

    impl Clone for TokenType {
        fn clone(&self) -> TokenType {
            match self {
                TokenType::Static => TokenType::Static,
                TokenType::Variable => TokenType::Variable,
            }
        }
    }

    impl Clone for Constraint {
        fn clone(&self) -> Constraint {
            match self {
                Constraint::Default => Constraint::Default,
                Constraint::Wildcard => Constraint::Wildcard,
                Constraint::Regex(regex) => Constraint::Regex(Regex::new(regex.as_str()).unwrap()),
            }
        }
    }

    fn cmp_tokens(a: Token, b: Token) -> Result<(), String> {
        if a.token_type != b.token_type {
            println!("{a}\n{b}");
            return Err(String::from("Token types do not match."))
        }

        if a.slice != b.slice {
            println!("{a}\n{b}");
            return Err(String::from("Token slices do not match."))
        }

        if a.range != b.range {
            println!("{a}\n{b}");
            return Err(String::from("Token ranges do not match."))
        }

        if let Constraint::Regex(a) = a.constraint
            && let Constraint::Regex(b) = b.constraint
        {
            if a.as_str() != b.as_str() {
                return Err(String::from("Token regex constraints do not match."))
            }
        }

        Ok(())
    }

    fn cmp_token_arr(a: Vec<Token>, b: Vec<Token>) -> Result<(), String> {
        for (i, token_a) in a.iter().enumerate() {
            cmp_tokens(token_a.clone(), b[i].clone())?
        }

        Ok(())
    }

    #[test]
    fn test_tokenizer() -> Result<(), String> {
        let segment = "test";
        let mut tokenizer = SegmentTokenizer::new(segment);
        cmp_token_arr(
            vec![Token::new_stat(0..segment.len(), segment)],
            tokenizer.tokenize(),
        )
        // cmp_tokens(Token::new_stat(0..segment.len(), segment), vec[0].clone())

        // //
        //
        // let segment = "{var}";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![Token::new_var(0..segment.len(), segment)];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // let segment = "{var_1}";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![Token::new_var(0..segment.len(), segment)];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // let segment = "test-{var1}";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![
        //     Token::new_stat(0..5, segment),
        //     Token::new_var(5..segment.len(), segment),
        // ];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // let segment = "{var}-end";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![
        //     Token::new_var(0..5, segment),
        //     Token::new_stat(5..segment.len(), segment),
        // ];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // let segment = "test-{var1}-t2";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![
        //     Token::new_stat(0..5, segment),
        //     Token::new_var(5..11, segment),
        //     Token::new_stat(11..segment.len(), segment),
        // ];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // let segment = "test-{var1}t2{var_2}";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![
        //     Token::new_stat(0..5, segment),
        //     Token::new_var(5..11, segment),
        //     Token::new_stat(11..13, segment),
        //     Token::new_var(13..segment.len(), segment),
        // ];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // let segment = "test-{var1}t2{var_2}-end";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![
        //     Token::new_stat(0..5, segment),
        //     Token::new_var(5..11, segment),
        //     Token::new_stat(11..13, segment),
        //     Token::new_var(13..20, segment),
        //     Token::new_stat(20..segment.len(), segment),
        // ];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // // Treat open brace as static if end of str
        // let segment = "test{";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![Token::new_stat(0..5, segment)];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // // Treat empty curly braces as static chars
        // let segment = "test{}";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![Token::new_stat(0..6, segment)];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // // Treat close brace as static if end of str
        // let segment = "test}";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![Token::new_stat(0..5, segment)];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // let segment = "{}";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![Token::new_stat(0..2, segment)];
        // assert_eq!(expect, tokenizer.tokenize());
        //
        // //
        //
        // // todo panic if we see something like this?
        // let segment = "{{}";
        // let mut tokenizer = SegmentTokenizer::new(segment);
        // let expect = vec![Token::new_stat(0..2, segment)];
        // assert_eq!(expect, tokenizer.tokenize());
    }
}
