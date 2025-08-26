use peek_again::{Peekable, PeekableIterator};
use std::{mem, str::Chars};

use crate::keywords::{self, IntToken};

#[derive(Debug)]
pub enum Token {
    Lf,
    Number(f32),
    Int(IntToken),
    Errors(Vec<String>),
}

pub struct Lexer<'src> {
    src: Peekable<Chars<'src>>,
    og_src: &'src str,
    errors: Vec<String>,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Lexer {
            src: src.chars().peek_again(),
            og_src: src,
            errors: Vec::new(),
        }
    }

    fn add_error(&mut self, error: impl ToString) {
        self.errors.push(error.to_string());
    }

    #[inline(always)]
    fn next_char(&mut self) -> Option<char> {
        let c = self.src.next();
        if c.is_some() {
            self.og_src = &self.og_src[1..];
        }
        c
    }

    #[inline(always)]
    fn peek_char(&mut self) -> Option<char> {
        self.src.peek().get().copied()
    }

    #[inline(always)]
    fn at_eof(&mut self) -> bool {
        self.peek_char().is_none()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(c) if c.is_whitespace() && c != '\n') {
            self.next_char();
        }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        if self.at_eof() {
            if self.errors.is_empty() {
                return None;
            } else {
                return Some(Token::Errors(mem::take(&mut self.errors)));
            }
        }

        match self.peek_char().unwrap() {
            '\n' => {
                self.next_char();
                Some(Token::Lf)
            }
            '#' => {
                self.next_char();
                while matches!(self.peek_char(), Some(c) if c != '\n') {
                    self.next_char();
                }
                self.next_char();
                Some(Token::Lf)
            }
            ',' => {
                self.next_char();
                self.next()
            }
            c if c.is_numeric() => {
                self.next_char();
                let mut ident = String::from(c);
                while let Some(c) = self.peek_char() {
                    if c.is_numeric() || c == '.' || c.eq_ignore_ascii_case(&'e') {
                        ident.push(c);
                    } else {
                        break;
                    }
                    self.next_char();
                }

                if let Ok(n) = ident.parse::<f32>() {
                    Some(Token::Number(n))
                } else {
                    self.add_error(format!("could not parse number {ident}"));
                    while matches!(self.peek_char(), Some(c) if c != '\n') {
                        self.next_char();
                    }
                    self.next_char();
                    Some(Token::Lf)
                }
            }
            _ => {
                let (token, skip) = if let Some((token, skip)) = keywords::make_token(self.og_src) {
                    (token, skip)
                } else {
                    let mut string = String::new();
                    while matches!(self.peek_char(), Some(c) if c != '\n') {
                        string.push(self.next_char().unwrap());
                    }
                    self.add_error(format!("could not parse \"{string}\". ignoring"));
                    self.next_char();
                    return Some(Token::Lf);
                };
                self.src.nth(skip - 1);
                self.og_src = &self.og_src[skip..];

                match token {
                    IntToken::Then => Some(Token::Lf),
                    tok => Some(Token::Int(tok)),
                }
            }
        }
    }
}
