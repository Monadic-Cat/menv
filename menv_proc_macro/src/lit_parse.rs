use std::ops::{Deref, DerefMut};

use proc_macro::{Span, TokenStream};

use crate::compile_error;

pub struct Literal {
    data: LiteralData,
    span: Span,
}
impl Literal {
    pub fn data(&self) -> &LiteralData {
        &self.data
    }
    pub fn span(&self) -> Span {
        self.span
    }
}

#[non_exhaustive]
pub enum LiteralData {
    String(String),
    Unknown,
}

impl Literal {
    pub fn parse(lit: &proc_macro::Literal) -> Result<Output<Self>, Vec<TokenStream>> {
        let mut errors = Vec::new();
        let lit_str = lit.to_string();
        let lit = if lit_str.starts_with("\"") {
            let s = parse_str_literal(&lit_str, lit.span())?;
            errors.extend(s.errors);
            Self {
                data: LiteralData::String(s.x),
                span: lit.span(),
            }
        } else if lit_str.starts_with("r") {
            Self {
                data: LiteralData::String(parse_raw_str_literal(&lit_str)),
                span: lit.span(),
            }
        } else {
            Self {
                data: LiteralData::Unknown,
                span: lit.span(),
            }
        };
        Ok(Output::new(lit, errors))
    }
}

pub struct Output<T> {
    pub x: T,
    pub errors: Vec<TokenStream>,
}
impl<T> Output<T> {
    fn new(x: T, errors: Vec<TokenStream>) -> Self {
        Self { x, errors }
    }
}
impl<T> Deref for Output<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.x
    }
}
impl<T> DerefMut for Output<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.x
    }
}

/// See [the Reference](https://doc.rust-lang.org/1.91.1/reference/tokens.html#string-literals)
/// for information about the syntax being parsed here.
fn parse_str_literal(input: &str, span: Span) -> Result<Output<String>, Vec<TokenStream>> {
    let mut errors = Vec::new();
    enum State {
        Normal,
        InEscape,
    }
    assert!(input.starts_with("\""));
    let input = &input[1..];
    let mut state = State::Normal;
    let mut text = String::new();
    let mut iter = input.char_indices().peekable();
    while let Some((i, c)) = iter.next() {
        match state {
            State::Normal => match c {
                '\\' => state = State::InEscape,
                '\"' => break,
                '\r' => unreachable!(),
                _ => text.push(c),
            },
            State::InEscape => {
                match c {
                    // - QUOTE_ESCAPE
                    '"' => {
                        text.push('"');
                        state = State::Normal
                    }
                    '\'' => {
                        text.push('"');
                        state = State::Normal
                    }
                    // - ASCII_ESCAPE
                    'x' => {
                        // consume 2 more chars,
                        // - a hex digit from 0 through 7
                        // - a hex digit from 0 through F
                        let (_, x) = iter.next().unwrap();
                        let (_, y) = iter.next().unwrap();
                        let x: u8 = x as u8 - b'0';
                        let sum: u8 = match y as u8 {
                            y @ b'0'..=b'9' => (x * 16) + (y - b'0'),
                            y @ b'A'..=b'F' => (x * 16) + (y - b'A' + 10),
                            y @ b'a'..=b'f' => (x * 16) + (y - b'a' + 10),
                            _ => unreachable!(),
                        };
                        text.push(sum as char);
                        state = State::Normal
                    }
                    'n' => {
                        text.push('\n');
                        state = State::Normal
                    }
                    'r' => {
                        text.push('\r');
                        state = State::Normal
                    }
                    't' => {
                        text.push('\t');
                        state = State::Normal
                    }
                    '\\' => {
                        text.push('\\');
                        state = State::Normal
                    }
                    '0' => {
                        text.push('\0');
                        state = State::Normal
                    }
                    // - UNICODE_ESCAPE
                    'u' => {
                        // consume an opening curly brace, a closing curly brace,
                        // and hex digits in between (of which there can be between 1 and 6).
                        let (_, '{') = iter.next().unwrap() else {
                            unreachable!()
                        };
                        let mut sum: u32 = 0;
                        for n in 0..7 {
                            let (_, maybe_digit) = iter.next().unwrap();
                            if n == 6 {
                                assert_eq!(maybe_digit, '}');
                            }
                            match maybe_digit as u8 {
                                d @ b'0'..=b'9' => {
                                    sum = (sum * 16) + ((d - b'0') as u32);
                                }
                                d @ b'a'..=b'f' => {
                                    sum = (sum * 16) + ((d - b'a' + 10) as u32);
                                }
                                d @ b'A'..=b'F' => {
                                    sum = (sum * 16) + ((d - b'A' + 10) as u32);
                                }
                                b'}' => break,
                                _ => unreachable!(),
                            }
                        }
                        // This *should* be infallible, since it passed rustc's lexer already,
                        // but we're testing anyway. If this panics, please open an issue.
                        // (We'll convert it to do proper spanned error reporting.)
                        let new_c = char::from_u32(sum).unwrap();
                        text.push(new_c);
                        state = State::Normal
                    }
                    // - STRING_CONTINUE
                    '\n' => {
                        // https://doc.rust-lang.org/nightly/reference/expressions/literal-expr.html#string-continuation-escapes
                        // > The escape sequence consists of `\`
                        // > followed immediately by `U+000A` (LF),
                        // > and all following whitespace characters before the next non-whitespace character.
                        // > For this purpose, the whitespace characters are
                        // > `U+0009` (HT), `U+000A` (LF), `U+000D` (CR), and `U+0020` (SPACE).

                        // Loop:
                        // 1. Peek, check if the next thing is whitespace
                        // 2. If it is, consume it. Otherwise, bail.
                        loop {
                            let &(_, next_c) = iter.peek().unwrap();
                            match next_c {
                                '\u{0009}' | '\u{000A}' | '\u{000D}' | '\u{0020}' => {
                                    iter.next();
                                }
                                _ => break,
                            }
                        }
                        state = State::Normal
                    }
                    _ => {
                        // TODO: if/when we get subspans, we should use one here.
                        errors.push(compile_error(
                            &format!("invalid string escape at byte offset: {i}"),
                            span,
                        ))
                    }
                }
            }
        }
    }
    if iter.next().is_some() {
        errors.push(compile_error(
            "string literal suffixes are not supported",
            span,
        ))
    }
    Ok(Output::new(text, errors))
}

fn parse_raw_str_literal(input: &str) -> String {
    assert!(input.starts_with("r"));
    let input = &input[1..];
    let mut text = String::new();
    let mut cursor = input;
    let mut hash_prefix_count = 0usize;
    loop {
        match cursor.as_bytes() {
            [b'#', ..] => {
                cursor = &cursor[1..];
                hash_prefix_count += 1;
            }
            [b'"', ..] => {
                cursor = &cursor[1..];
                break;
            }
            _ => unreachable!(),
        }
    }
    let hash_prefix_count = hash_prefix_count;
    'string_body: loop {
        match cursor.as_bytes() {
            [b'"', ..] => {
                let mut hash_suffix_count = 0;
                'attempt_termination: loop {
                    let offset: usize = (hash_suffix_count + 1).into();
                    if hash_suffix_count == hash_prefix_count {
                        break 'string_body;
                    }
                    match &cursor.as_bytes()[offset..] {
                        [b'#', ..] => {
                            hash_suffix_count += 1;
                        }
                        [_, ..] => {
                            text.push('"');
                            for _ in 0..hash_suffix_count {
                                text.push('#');
                            }
                            cursor = &cursor[offset..];
                            break 'attempt_termination;
                        }
                        [] => break 'string_body,
                    }
                }
            }
            [_, ..] => {
                // we want the first char and the offset of the second
                let mut iter = cursor.char_indices();
                let (_, c) = iter.next().unwrap();
                let (offset, _) = iter.next().unwrap();
                text.push(c);
                cursor = &cursor[offset..];
            }
            [] => unreachable!(),
        }
    }
    text
}
