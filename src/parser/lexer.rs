use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::CharIndices;
use itertools::Itertools;

use crate::parser::error::Error;

/// Token returned by the lexers
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Identifier(&'a str),
    MacroIdentifier(&'a str),
    OperatorIdentifier(&'a str),
    StringLiteral(String),
    IntLiteral(&'a str),
    RealLiteral(&'a str),
    Symbol(&'a str),
}

/// A concrete lexer
#[derive(Clone)]
pub struct Lexer<'i> {
    /// The source string.
    source: &'i str,
    /// The unconsumed input.
    /// We need to peek a maximum of 2 at a time:
    ///     0.0 => RealLiteral
    ///     0.a => IntLiteral . Identifier
    input: Peekable<CharIndices<'i>>,

    /// For every string we are in, the amount of opened (.
    /// When the last is closed with ), the last one is popped.
    string_context: Vec<usize>,

    /// Sometimes, we need to emit two tokens at once.
    /// e.g. when we find " in a string, we need to emit the current string part,
    /// as well as the " token itself.
    next_planned: Option<<Self as Iterator>::Item>,
}

impl<'i> Lexer<'i> {
    pub fn new(source: &'i str) -> Self {
        Self {
            source,
            input: source.char_indices().peekable(),
            string_context: vec![],
            next_planned: None,
        }
    }
}

impl<'i> Iterator for Lexer<'i> {
    type Item = Result<(usize, Token<'i>, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // This function runs just like the default lalrpop tokenizer:
        // It tests all tokens for a regex match and returns the longest.

        // The exceptions are:
        // 1) \( in string literals emit the previous part as StringContext and the string context is exited temporarily.
        //    When the \( is closed with ), the string context continues.
        // 2) Encountered comments are stored elsewhere for transpilation (WIP).

        // Some complexity is added to avoid doing the same work more than once - e.g.
        //  not testing every string for symbol equality, but only when the current identifier
        //  is a string of the same length.

        if let Some(next) = self.next_planned.take() {
            return Some(next);
        }

        if let Some(0) = self.string_context.last() {
            return self.scan_string_part();
        }

        return self.scan_normal_token();
    }
}

impl<'i> Lexer<'i> {
    fn scan_normal_token(&mut self) -> Option<<Self as Iterator>::Item> {
        loop {
            // We are in normal token scanning mode.
            let Some((start, ch)) = self.input.next() else {
                // End of file
                return None;
            };

            // Skip over whitespace
            if ch.is_ascii_whitespace() {
                continue;
            }

            if matches!(ch, '"') {
                self.string_context.push(0);
                return self.make_token_from(start, Token::Symbol);
            }

            if matches!(ch, '{' | '}' | '(' | ')' | '[' | ']' | ':' | '@' | '\'' | ',' | ';') {
                if let Some((_, ':')) = self.input.peek() {
                    // Consume :
                    self.input.next();
                    self.make_token_from(start, Token::Symbol);
                }

                // If it's ( or ), we need to modify the current string context.
                match ch {
                    '(' => _ = self.string_context.last_mut().map(|i| *i += 1),
                    ')' => _ = self.string_context.last_mut().map(|i| *i -= 1),
                    _ => {}
                }

                return self.make_token_from(start, Token::Symbol)
            }

            if matches!(ch, '0'..='9') {
                self.input.by_ref().peeking_take_while(|(_, ch)| matches!(ch, '0'..='9')).count() + 1;

                let Some((dot_start, '.')) = self.input.peek().cloned() else {
                    return self.make_token_from(start, Token::IntLiteral);
                };

                // Skip .
                self.input.next();

                if self.input.peeking_take_while(|(_, ch)| matches!(ch, '0'..='9')).count() > 0 {
                    // We found at least one digit! Skip all digits.
                    return self.make_token_from(start, Token::RealLiteral)
                } else {
                    // The next is a dot (already consumed)
                    self.next_planned = self.make_token_from(dot_start, Token::Symbol);
                    return Some(Ok((start, Token::IntLiteral(&self.source[start..dot_start]), dot_start)));
                }
            }

            if matches!(ch, 'a'..='z' | 'A'..='Z' | '_' | '$' | '#') {
                let len = self.input.by_ref().peeking_take_while(|(_, ch)| ch.is_alphanumeric() || matches!(ch, '_' | '$' | '#')).count() + 1;

                if let Some((_, '!')) = self.input.peek() {
                    let macro_token = self.make_token_from(start, Token::MacroIdentifier);
                    self.input.next();  // Skip !
                    return macro_token;
                };

                let end = peek_pos(&mut self.input, self.source);
                let slice = unsafe { self.source.get_unchecked(start..end) };

                if match len {
                    7 => matches!(slice, "declare"),
                    6 => matches!(slice, "return"),
                    5 => matches!(slice, "trait"),
                    3 => matches!(slice, "let" | "var" | "upd" | "def"),
                    2 => matches!(slice, "is"),
                    _ => false,
                } {
                    return Some(Ok((start, Token::Symbol(slice), end)));
                };

                return Some(Ok((start, Token::Identifier(slice), end)));
            }
            else if matches!(ch, '!' | '+' | '\\' | '-' | '*' | '/' | '&' | '%' | '=' | '>' | '<' | '|' | '.' | '^' | '?') {
                if let Some((_, '-')) = self.input.peek() {
                    // Skip comment
                    // TODO We should collect the comment and put it somewhere
                    self.input.by_ref().take_while(|(_, ch)| ch != &'\n').count();
                    continue;
                }

                let len = self.input.by_ref().peeking_take_while(|(_, ch)| matches!(ch, '!' | '+' | '\\' | '-' | '*' | '/' | '&' | '%' | '=' | '>' | '<' | '|' | '.' | '^' | '?' | '_')).count() + 1;

                let end = peek_pos(&mut self.input, self.source);
                let slice = unsafe { self.source.get_unchecked(start..end) };

                if match len {
                    2 => matches!(slice, "->"),
                    1 => matches!(ch, '=' | '.' | '!'),
                    _ => false,
                } {
                    return Some(Ok((start, Token::Symbol(slice), end)));
                };

                return Some(Ok((start, Token::OperatorIdentifier(slice), end)));
            }

            return Some(Err(Error(format!("Unexpected Character: {}", ch))));
        }
    }

    fn scan_string_part(&mut self) -> Option<<Self as Iterator>::Item> {
        // We are in a string literal!
        // Let's collect all the characters we have.
        let string_part_start = peek_pos(&mut self.input, self.source);
        let mut string_builder_preemptive_end_chars = 0;
        let mut builder = String::new();

        // Advance until " or \(
        loop {
            let Some((pos, ch)) = self.input.next() else {
                // Unterminated string.
                // Handled in lalrpop, due to missing "
                break;
            };

            match ch {
                '"' => {
                    // End of string. Plan to emit a ", but first emit the current literal, if any.
                    self.string_context.pop();
                    self.next_planned = self.make_token_from(pos, Token::Symbol);
                    string_builder_preemptive_end_chars = 1;
                    break;
                },
                '\\' => {
                    // Escape next character
                    let Some((pos, ch)) = self.input.next() else {
                        // Unterminated string, just after a \.
                        // Handled in lalrpop, due to missing "
                        break;
                    };

                    match ch {
                        '(' => {
                            // Starting a struct! Plan to emit (, but first emit the current literal, if any.
                            *self.string_context.last_mut().unwrap() += 1;
                            self.next_planned = self.make_token_from(pos, Token::Symbol);
                            string_builder_preemptive_end_chars = 2;
                            break;
                        }
                        '\\' | '"' => builder.push(ch),
                        '0' => builder.push('\0'),
                        'n' => builder.push('\n'),
                        't' => builder.push('\t'),
                        'r' => builder.push('\r'),
                        _ => return Some(Err(Error(format!("Invalid escape sequence in string literal: {}", ch))))
                    }
                }
                // Normal character.
                _ => builder.push(ch),
            }
        }

        // If the builder is not empty, emit its contents.
        if !builder.is_empty() {
            return Some(Ok((
                string_part_start,
                Token::StringLiteral(builder),
                // The string part may end one or two chars early of where we are now.
                peek_pos(&mut self.input, self.source) - string_builder_preemptive_end_chars
            )))
        }

        // If we have something planned, emit that now.
        // Otherwise, it's eof.
        return self.next_planned.take();
    }

    fn make_token_from(&mut self, start: usize, token: fn(&'i str) -> Token<'i>) -> Option<<Self as Iterator>::Item> {
        let end = peek_pos(&mut self.input, self.source);
        let slice = unsafe { self.source.get_unchecked(start..end) };
        Some(Ok((start, token(slice), end)))
    }
}

#[inline]
fn peek_pos(input: &mut Peekable<CharIndices>, full_str: &str) -> usize {
    match input.peek() {
        Some((pos, _)) => *pos,
        None => full_str.len(),
    }
}

impl<'i> Display for Token<'i> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "{}", s),
            Token::MacroIdentifier(s) => write!(f, "{}", s),
            Token::OperatorIdentifier(s) => write!(f, "{}", s),
            Token::IntLiteral(s) => write!(f, "{}", s),
            Token::RealLiteral(s) => write!(f, "{}", s),
            Token::Symbol(s) => write!(f, "{}", s),
            Token::StringLiteral(s) => write!(f, "{}", s),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
