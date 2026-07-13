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
            // todo It may be possible that self.state_start is greater than i,
            // todo if so, possibly skip until i == self.state_start

            match char {
                '{' => {
                    // If this is the last character, break loop and
                    // consider this curly brace a static character
                    if i == self.segment.len() - 1 {
                        break;
                    }

                    if self.state == State::InCurly {
                        // We were already in a curly brace
                        let prev = tokens.last_mut();
                        if let Some(token) = prev {
                            match token.token_type {
                                // Append to the previous token
                                TokenType::Static => {
                                    token.range.end = i;
                                    token.slice = &self.segment[token.range.clone()];
                                }
                                // Previous token already closed, push new static token instead
                                TokenType::Variable => {
                                    tokens.push(Token::new_var(self.state_start..i, &self.segment));
                                }
                            }
                        } else {
                            // No prev token
                            tokens.push(Token::new_stat(self.state_start..i, &self.segment));
                        }

                        self.state_start = i;

                        continue;
                    }

                    // Check if previous state (State::Default) has chars to push as static string
                    if i > self.state_start {
                        tokens.push(Token::new_stat(self.state_start..i, &self.segment));
                    }

                    self.change_state(State::InCurly, i);
                }
                '}' => {
                    if self.state == State::Default {
                        // if prev token and token is static, push to it
                        // if prev token and token is var, create new token
                        // if no prev token, create new token
                        let prev = tokens.last_mut();
                        if let Some(token) = prev {
                            match token.token_type {
                                // Append to the previous token
                                TokenType::Static => {
                                    token.range.end = i + 1;
                                    token.slice = &self.segment[token.range.clone()];
                                }
                                // Previous token already closed, push new static token instead
                                TokenType::Variable => {
                                    tokens.push(Token::new_var(
                                        self.state_start..i + 1,
                                        &self.segment,
                                    ));
                                }
                            }
                        } else {
                            // No prev token
                            tokens.push(Token::new_stat(self.state_start..i + 1, &self.segment));
                        }

                        self.state_start = i + 1;

                        continue;
                    }

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
                                    token.range.end = i + 1;
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

#[derive(Debug, PartialEq)]
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

    fn cmp_tokens(a: Token, b: Token, calling_line: String) -> Result<(), String> {
        if a.token_type != b.token_type {
            println!("Left:  {a}\nRight: {b}");
            return Err(String::from(format!(
                "Token types do not match. {calling_line}"
            )));
        }

        if a.slice != b.slice {
            println!("Left:  {a}\nRight: {b}");
            return Err(String::from(format!(
                "Token slices do not match. {calling_line}"
            )));
        }

        if a.range != b.range {
            println!("Left:  {a}\nRight: {b}");
            return Err(String::from(format!(
                "Token ranges do not match. {calling_line}"
            )));
        }

        if let Constraint::Regex(a) = a.constraint
            && let Constraint::Regex(b) = b.constraint
        {
            if a.as_str() != b.as_str() {
                println!("Left:  {a}\nRight: {b}");
                return Err(String::from(format!(
                    "Token regex constraints do not match. {calling_line}"
                )));
            }
        }

        Ok(())
    }

    fn cmp_token_arr(a: Vec<Token>, b: Vec<Token>, calling_line: String) -> Result<(), String> {
        if a.len() != b.len() {
            println!("Left  ======================>");
            for (i, token_a) in a.iter().enumerate() {
                println!("{token_a}");
            }
            println!("Right ======================>");
            for (i, token_b) in b.iter().enumerate() {
                println!("{token_b}");
            }
            println!("End   ======================>");

            return Err(format!("Token amounts do not match. {calling_line}"));
        }

        for (i, token_a) in a.iter().enumerate() {
            if let Err(error) = cmp_tokens(token_a.clone(), b[i].clone(), calling_line.clone()) {
                return Err(format!("Segment depth({}): {}", i + 1, error));
            }
            cmp_tokens(token_a.clone(), b[i].clone(), calling_line.clone())?
        }

        Ok(())
    }

    #[test]
    fn test_single_stat() -> Result<(), String> {
        let segment = "test";
        let mut tokenizer = SegmentTokenizer::new(segment);
        cmp_token_arr(
            vec![Token::new_stat(0..segment.len(), segment)],
            tokenizer.tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn test_single_var() -> Result<(), String> {
        let segment = "{var}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_var(0..segment.len(), segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_underscore_number_in_var() -> Result<(), String> {
        let segment = "{var_1}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_var(0..segment.len(), segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_dash_in_stat() -> Result<(), String> {
        let segment = "test-{var1}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::new_stat(0..5, segment),
            Token::new_var(5..segment.len(), segment),
        ];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_var_to_stat() -> Result<(), String> {
        let segment = "{var}-end";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::new_var(0..5, segment),
            Token::new_stat(5..segment.len(), segment),
        ];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_stat_var_stat() -> Result<(), String> {
        let segment = "test-{var1}-t2";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::new_stat(0..5, segment),
            Token::new_var(5..11, segment),
            Token::new_stat(11..segment.len(), segment),
        ];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_stat_var_stat_var() -> Result<(), String> {
        let segment = "test-{var1}t2{var_2}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::new_stat(0..5, segment),
            Token::new_var(5..11, segment),
            Token::new_stat(11..13, segment),
            Token::new_var(13..segment.len(), segment),
        ];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_stat_var_stat_var_stat() -> Result<(), String> {
        let segment = "test-{var1}t2{var_2}-end";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::new_stat(0..5, segment),
            Token::new_var(5..11, segment),
            Token::new_stat(11..13, segment),
            Token::new_var(13..20, segment),
            Token::new_stat(20..segment.len(), segment),
        ];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_stranded_open_curly() -> Result<(), String> {
        // Treat open brace as static if end of str
        let segment = "test{";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_stat(0..5, segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_stranded_close_curly() -> Result<(), String> {
        // Treat close brace as static if end of str
        let segment = "test}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_stat(0..5, segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_empty_curly() -> Result<(), String> {
        let segment = "{}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_stat(0..2, segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_empty_curly_with_leading_stat() -> Result<(), String> {
        // Treat empty curly braces as static chars
        let segment = "test{}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_stat(0..6, segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_empty_curly_with_trailing_stat() -> Result<(), String> {
        // Treat empty curly braces as static chars
        let segment = "{}test";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_stat(0..6, segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_stranded_open_curly_into_var() -> Result<(), String> {
        let segment = "{test{actual_var}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::new_stat(0..5, segment),
            Token::new_var(5..segment.len(), segment),
        ];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_stranded_open_curly_into_var_2() -> Result<(), String> {
        let segment = "test{test{actual_var}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::new_stat(0..9, segment),
            Token::new_var(9..segment.len(), segment),
        ];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_stranded_open_curly_into_var_3() -> Result<(), String> {
        let segment = "test{{actual_var}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![
            Token::new_stat(0..5, segment),
            Token::new_var(5..segment.len(), segment),
        ];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_stranded_open_into_empty_curlies() -> Result<(), String> {
        let segment = "{{}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_stat(0..3, segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!())
    }

    #[test]
    fn test_empty_curlies_wrapped_by_stranded_curlies() -> Result<(), String> {
        let segment = "{{}}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_stat(0..4, segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!());

        Ok(())
    }

    #[test]
    fn test_empty_curlies_wrapped_by_stranded_curlies_2() -> Result<(), String> {
        let segment = "{{{}}}";
        let mut tokenizer = SegmentTokenizer::new(segment);
        let expect = vec![Token::new_stat(0..4, segment)];
        cmp_token_arr(expect, tokenizer.tokenize(), get_line!());

        Ok(())
    }

    macro_rules! get_line {
        () => {
            format!("{}:{}:{}", file!(), line!(), column!())
        };
    }

    use get_line;
}
