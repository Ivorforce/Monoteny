use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::CharIndices;
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
    source: &'i str,
    input: Peekable<CharIndices<'i>>,

    string_context: Vec<(usize, i32)>,
    next_planned: Option<(usize, Token<'i>, usize)>,
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
            return Some(Ok(next));
        }

        // Are we in a base string?
        let (string_start, distance_from_string) = self.string_context.last().cloned().unwrap_or((0, -1));
        if distance_from_string == 0 {
            let mut builder = String::new();

            // Advance until " or \(
            loop {
                if let Some((pos, ch)) = self.input.next() {
                    match ch {
                        '"' => {
                            // End of string. Emit the constant, but plan to emit a " next.
                            self.string_context.pop();
                            let end = self.peek_next_pos(self.input.clone());
                            let slice = unsafe { self.source.get_unchecked(pos..end) };
                            self.next_planned = Some((pos, Token::Symbol(slice), end));
                            break;
                        },
                        '\\' => {
                            // Escape next character
                            if let Some((pos, ch)) = self.input.next() {
                                match ch {
                                    '(' => {
                                        // We are in a struct! Emit the constant, but plan to emit a ( next.
                                        self.string_context.last_mut().unwrap().1 += 1;
                                        let end = self.peek_next_pos(self.input.clone());
                                        let slice = unsafe { self.source.get_unchecked(pos..end) };
                                        self.next_planned = Some((pos, Token::Symbol(slice), end));
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
                            else {
                                // Skip over the escape char and push nothing.
                                // Technically a mistake but it might be WIP.
                                break;
                            }
                        }
                        _ => builder.push(ch),
                    }
                }
                else {
                    break;
                }
            }

            // If the string is empty it's not worth emitting a StringLiteral for.
            if !builder.is_empty() {
                let end = string_start + builder.as_bytes().len();
                return Some(Ok((string_start, Token::StringLiteral(builder), end)))
            }

            // Either way we definitely have a token planned. Either " or (
            return Some(Ok(self.next_planned.take().unwrap()))
        }

        loop {
            if let Some((start, ch)) = self.input.next() {
                // Skip over whitespace
                if ch.is_ascii_whitespace() {
                    continue;
                }

                if matches!(ch, '"') {
                    self.string_context.push((start, 0));
                    let end = self.peek_next_pos(self.input.clone());
                    let slice = unsafe { self.source.get_unchecked(start..end) };
                    return Some(Ok((start, Token::Symbol(slice), end)))
                }

                if let Some(result) = self.match_slice(self.input.clone(), start, 2, |str| matches!(str, "--")) {
                    // Skip comment
                    Self::advance_while(&mut self.input, |ch| ch != '\n');
                    self.input.next();  // Skip the newline too.
                    continue;
                }

                if let Some(result) = self.match_slice(self.input.clone(), start, 2, |str| matches!(str, "::")) {
                    self.input.next();
                    return Some(Ok(result))
                }

                if matches!(ch, '{' | '}' | '(' | ')' | '[' | ']' | ':' | '@' | '\'' | ',' | ';') {
                    if distance_from_string >= 0 {
                        if ch == '(' {
                            self.string_context.last_mut().unwrap().1 += 1;
                        }
                        else if ch == ')' {
                            self.string_context.last_mut().unwrap().1 -= 1;
                        }
                    }

                    let end = self.peek_next_pos(self.input.clone());
                    let slice = unsafe { self.source.get_unchecked(start..end) };
                    return Some(Ok((start, Token::Symbol(slice), end)))
                }

                if matches!(ch, '0'..='9') {
                    let mut input = self.input.clone();
                    let mut len = 1 + Self::advance_while(&mut input, |ch| matches!(ch, '0'..='9'));

                    let is_float = match input.peek() {
                        Some((_, '.')) => {
                            input.next();  // Skip dot.
                            let len_plus = Self::advance_while(&mut input, |ch| matches!(ch, '0'..='9'));
                            if len_plus > 0 {
                                len += 1 + len_plus;
                                true
                            }
                            else {
                                false
                            }
                        }
                        _ => false,
                    };

                    advance(&mut self.input, len - 1);
                    let end = self.peek_next_pos(input);
                    let slice = unsafe { self.source.get_unchecked(start..end) };
                    return Some(Ok((start, if is_float { Token::RealLiteral(slice) } else { Token::IntLiteral(slice)}, end)));
                }

                if matches!(ch, 'a'..='z' | 'A'..='Z' | '_' | '$' | '#') {
                    let mut input = self.input.clone();
                    let len = 1 + Self::advance_while(&mut input, |ch| ch.is_alphanumeric() || matches!(ch, '_' | '$' | '#'));
                    advance(&mut self.input, len - 1);

                    if let Some((_, '!')) = input.peek() {
                        self.input.next();  // Skip !
                        let end = self.peek_next_pos(input);
                        let slice = unsafe { self.source.get_unchecked(start..end) };
                        return Some(Ok((start, Token::MacroIdentifier(slice), end)));
                    };

                    let end = self.peek_next_pos(input);
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
                    let mut input = self.input.clone();
                    let len = 1 + Self::advance_while(&mut input, |ch| matches!(ch, '!' | '+' | '\\' | '-' | '*' | '/' | '&' | '%' | '=' | '>' | '<' | '|' | '.' | '^' | '?' | '_'));

                    advance(&mut self.input, len - 1);
                    let end = self.peek_next_pos(input);
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

                return Some(Err(Error(format!("Unrecognized Symbol: {}", ch))));
            }
            else {
                return None;
            }
        }
    }
}

impl<'i> Lexer<'i> {
    // Advances until the next peeked character does not match f anymore.
    fn advance_while(input: &mut Peekable<CharIndices>, f: fn(char) -> bool) -> i32 {
        let mut len = 0;

        loop {
            if let Some((pos, ch)) = input.peek() {
                if !f(*ch) {
                    return len;
                }
                len += 1;
                input.next();
            }
            else {
                return len;
            }
        };
    }
}

impl<'i> Lexer<'i> {
    #[inline]
    fn match_slice(&self, mut input: Peekable<CharIndices>, start: usize, len: i32, f: fn(&str) -> bool) -> Option<(usize, Token<'i>, usize)> {
        advance(&mut input, len - 1);
        let end = self.peek_next_pos(input);
        let slice = unsafe { self.source.get_unchecked(start..end) };
        return match f(slice) {
            true => Some((start, Token::Symbol(slice), end)),
            false => None,
        }
    }

    #[inline]
    fn peek_next_pos(&self, mut input: Peekable<CharIndices>) -> usize {
        match input.peek() {
            None => self.source.len(),
            Some((pos, _)) => *pos,
        }
    }
}

#[inline]
fn advance(indices: &mut impl Iterator, n: i32) {
    for _ in 0..n {
        indices.next();
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
