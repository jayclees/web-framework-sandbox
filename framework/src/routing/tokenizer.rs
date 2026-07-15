use regex::Regex;
use std::ops::Range;

#[derive(Debug)]
pub struct SegmentTokenizer {
    state: State,
    segment: String,
    state_start: usize,
}

impl SegmentTokenizer {
    pub fn new(segment: String) -> SegmentTokenizer {
        SegmentTokenizer {
            state: State::Default,
            segment,
            state_start: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        // see tokenizer::split_segments
        if self.segment == "" {
            return vec![Token::new_stat(0..0, "".to_owned())];
        }

        let mut tokens: Vec<Token> = vec![];

        for (i, char) in self.segment.clone().chars().enumerate() {
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
                            match token.constraint {
                                // Append to the previous token
                                Constraint::Static => {
                                    token.range.end = i;
                                    token.slice = self.segment[token.range.clone()].to_owned();
                                }
                                // Previous token already closed, push new static token instead
                                _ => {
                                    tokens.push(Token::new_var(
                                        self.state_start..i,
                                        self.segment[self.state_start..i].to_owned(),
                                    ));
                                }
                            }
                        } else {
                            // No prev token
                            tokens.push(Token::new_stat(
                                self.state_start..i,
                                self.segment[self.state_start..i].to_owned(),
                            ));
                        }

                        self.state_start = i;

                        continue;
                    }

                    // Check if previous state (State::Default) has chars to push as static string
                    if i > self.state_start {
                        tokens.push(Token::new_stat(
                            self.state_start..i,
                            self.segment[self.state_start..i].to_owned(),
                        ));
                    }

                    self.change_state(State::InCurly, i);
                }
                '}' => {
                    // If we find a closing curly brace, but we're already in a static state
                    if self.state == State::Default {
                        // if prev token and token is static, push to it
                        // if prev token and token is var, create new token
                        // if no prev token, create new token
                        let prev = tokens.last_mut();
                        if let Some(token) = prev {
                            match token.constraint {
                                // Append to the previous token
                                Constraint::Static => {
                                    token.range.end = i + 1;
                                    token.slice = self.segment[token.range.clone()].to_owned();
                                }
                                // Previous variable token already closed, push new static token instead
                                _ => {
                                    tokens.push(Token::new_stat(
                                        self.state_start..i + 1,
                                        self.segment[self.state_start..i + 1].to_owned(),
                                    ));
                                }
                            }
                        } else {
                            // No prev token
                            tokens.push(Token::new_stat(
                                self.state_start..i + 1,
                                self.segment[self.state_start..i + 1].to_owned(),
                            ));
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
                                if let Some(token) = tokens.last_mut()
                                    && token.constraint.is_static()
                                {
                                    // overwrite last token to include the last 2 chars "{}"
                                    token.range.end = i + 1;
                                    token.slice = self.segment[token.range.clone()].to_owned();
                                    self.change_state(State::Default, i + 1);
                                }
                            } else {
                                tokens.push(Token::new_var(
                                    self.state_start..i + 1,
                                    self.segment[self.state_start..i + 1].to_owned(),
                                ));
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
            tokens.push(Token::new_stat(
                self.state_start..self.segment.len(),
                self.segment[self.state_start..self.segment.len()].to_owned(),
            ));
        }

        tokens
    }

    fn change_state(&mut self, state: State, index: usize) {
        self.state = state;
        self.state_start = index;
    }
}

#[derive(Debug, PartialEq)]
enum State {
    Default,
    InCurly,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub range: Range<usize>,
    pub slice: String,
    pub constraint: Constraint,
}

impl Token {
    fn new_stat(range: Range<usize>, slice: String) -> Token {
        let clone = range.clone();
        Token {
            range: clone,
            slice,
            constraint: Constraint::Static,
        }
    }

    fn new_var(range: Range<usize>, slice: String) -> Token {
        let clone = range.clone();
        Token {
            range: clone,
            slice,
            constraint: Constraint::Default,
        }
    }

    pub fn constrain(&mut self, pattern: &'static str) {
        self.constraint = Constraint::Regex(Regex::new(pattern).unwrap())
    }

    pub fn wildcard(&mut self, enable: bool) {
        self.constraint = if enable {
            Constraint::Wildcard
        } else {
            Constraint::Default
        }
    }
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Static,
    Default, // default regex: .*
    Wildcard,
    Regex(Regex),
}

impl Constraint {
    pub fn is_static(&self) -> bool {
        match self {
            Constraint::Static => true,
            _ => false,
        }
    }

    pub fn is_default(&self) -> bool {
        match self {
            Constraint::Default => true,
            _ => false,
        }
    }

    pub fn is_wildcard(&self) -> bool {
        match self {
            Constraint::Wildcard => true,
            _ => false,
        }
    }

    pub fn is_regex(&self, pattern: &str) -> bool {
        match self {
            Constraint::Regex(regex) => regex.as_str() == pattern,
            _ => false,
        }
    }
}

impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        match other {
            Constraint::Static => self.is_static(),
            Constraint::Default => self.is_default(),
            Constraint::Wildcard => self.is_wildcard(),
            Constraint::Regex(regex) => self.is_regex(regex.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_line;
    use std::fmt::{Display, Formatter};

    #[test]
    fn empty_segment() -> Result<(), String> {
        let segment = "".to_owned();
        let expect = vec![Token::new_stat(0..0, "".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn single_stat() -> Result<(), String> {
        let segment = "test".to_owned();
        cmp_token_arr(
            vec![Token::new_stat(0..segment.len(), "test".to_owned())],
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn single_var() -> Result<(), String> {
        let segment = "{var}".to_owned();
        let expect = vec![Token::new_var(0..segment.len(), "{var}".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn underscore_number_in_var() -> Result<(), String> {
        let segment = "{var_1}".to_owned();
        let expect = vec![Token::new_var(0..segment.len(), "{var_1}".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn dash_in_stat() -> Result<(), String> {
        let segment = "test-{var1}".to_owned();
        let expect = vec![
            Token::new_stat(0..5, "test-".to_owned()),
            Token::new_var(5..segment.len(), "{var1}".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn var_to_stat() -> Result<(), String> {
        let segment = "{var}-end".to_owned();
        let expect = vec![
            Token::new_var(0..5, "{var}".to_owned()),
            Token::new_stat(5..segment.len(), "-end".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stat_var_stat() -> Result<(), String> {
        let segment = "test-{var1}-t2".to_owned();
        let expect = vec![
            Token::new_stat(0..5, "test-".to_owned()),
            Token::new_var(5..11, "{var1}".to_owned()),
            Token::new_stat(11..segment.len(), "-t2".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stat_var_stat_var() -> Result<(), String> {
        let segment = "test-{var1}t2{var_2}".to_owned();
        let expect = vec![
            Token::new_stat(0..5, "test-".to_owned()),
            Token::new_var(5..11, "{var1}".to_owned()),
            Token::new_stat(11..13, "t2".to_owned()),
            Token::new_var(13..segment.len(), "{var_2}".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stat_var_stat_var_stat() -> Result<(), String> {
        let segment = "test-{var1}t2{var_2}-end".to_owned();
        let expect = vec![
            Token::new_stat(0..5, "test-".to_owned()),
            Token::new_var(5..11, "{var1}".to_owned()),
            Token::new_stat(11..13, "t2".to_owned()),
            Token::new_var(13..20, "{var_2}".to_owned()),
            Token::new_stat(20..segment.len(), "-end".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stranded_open_curly() -> Result<(), String> {
        // Treat open brace as static if end of str
        let segment = "test{".to_owned();
        let expect = vec![Token::new_stat(0..5, "test{".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stranded_close_curly() -> Result<(), String> {
        // Treat close brace as static if end of str
        let segment = "test}".to_owned();
        let expect = vec![Token::new_stat(0..5, "test}".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn empty_curly() -> Result<(), String> {
        let segment = "{}".to_owned();
        let expect = vec![Token::new_stat(0..2, "{}".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn empty_curly_with_leading_stat() -> Result<(), String> {
        // Treat empty curly braces as static chars
        let segment = "test{}".to_owned();
        let expect = vec![Token::new_stat(0..6, "test{}".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn empty_curly_with_trailing_stat() -> Result<(), String> {
        // Treat empty curly braces as static chars
        let segment = "{}test".to_owned();
        let expect = vec![Token::new_stat(0..6, "{}test".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stranded_open_curly_into_var() -> Result<(), String> {
        let segment = "{test{actual_var}".to_owned();
        let expect = vec![
            Token::new_stat(0..5, "{test".to_owned()),
            Token::new_var(5..segment.len(), "{actual_var}".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stranded_open_curly_into_var_2() -> Result<(), String> {
        let segment = "test{test{actual_var}".to_owned();
        let expect = vec![
            Token::new_stat(0..9, "test{test".to_owned()),
            Token::new_var(9..segment.len(), "{actual_var}".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stranded_open_curly_into_var_3() -> Result<(), String> {
        let segment = "test{{actual_var}".to_owned();
        let expect = vec![
            Token::new_stat(0..5, "test{".to_owned()),
            Token::new_var(5..segment.len(), "{actual_var}".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stranded_open_curly_into_var_4() -> Result<(), String> {
        let segment = "test{test{actual_var}test}test}".to_owned();
        let expect = vec![
            Token::new_stat(0..9, "test{test".to_owned()),
            Token::new_var(9..21, "{actual_var}".to_owned()),
            Token::new_stat(21..31, "test}test}".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn stranded_open_into_empty_curlies() -> Result<(), String> {
        let segment = "{{}".to_owned();
        let expect = vec![Token::new_stat(0..3, "{{}".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn empty_curlies_wrapped_by_stranded_curlies() -> Result<(), String> {
        let segment = "{{}}".to_owned();
        let expect = vec![Token::new_stat(0..4, "{{}}".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn empty_curlies_wrapped_by_stranded_curlies_2() -> Result<(), String> {
        let segment = "{{{}}}".to_owned();
        let expect = vec![Token::new_stat(0..6, "{{{}}}".to_owned())];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    #[test]
    fn author_post_id_post_slug_segment() -> Result<(), String> {
        let segment = "{author}.{post_id}.{slug}".to_owned();
        let expect = vec![
            Token::new_var(0..8, "{author}".to_owned()),
            Token::new_stat(8..9, ".".to_owned()),
            Token::new_var(9..18, "{post_id}".to_owned()),
            Token::new_stat(18..19, ".".to_owned()),
            Token::new_var(19..segment.len(), "{slug}".to_owned()),
        ];
        cmp_token_arr(
            expect,
            SegmentTokenizer::new(segment).tokenize(),
            get_line!(),
        )
    }

    fn cmp_tokens(a: Token, b: Token, calling_line: String) -> Result<(), String> {
        if a.constraint != b.constraint {
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
            for token_a in a.iter() {
                println!("{token_a}");
            }
            println!("Right ======================>");
            for token_b in b.iter() {
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

    impl Display for Token {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Token {{ range: {}, slice: \"{}\", constraint: {} }}",
                format!("Range {{ {}..{} }}", self.range.start, self.range.end),
                self.slice,
                self.constraint,
            )
        }
    }

    impl Display for Constraint {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Constraint::Static => write!(f, "{}", "Constraint::Static"),
                Constraint::Default => write!(f, "{}", "Constraint::Default"),
                Constraint::Wildcard => write!(f, "{}", "Constraint::Wildcard"),
                Constraint::Regex(re) => {
                    write!(f, "{}", format!("Constraint::Regex(\"{}\")", re.as_str()))
                }
            }
        }
    }
}
