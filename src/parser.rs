use std::f32::consts::PI;

use crate::keywords::IntToken;
use peek_again::{Peekable, PeekableIterator};

use crate::{
    Command,
    lexer::{Lexer, Token},
};

pub struct Parser<'src> {
    lexer: Peekable<Lexer<'src>>,
    errors: Vec<String>,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Self {
        Parser {
            lexer: Lexer::new(src).peek_again(),
            errors: Vec::new(),
        }
    }

    fn add_error(&mut self, error: impl ToString) {
        self.errors.push(error.to_string());
    }

    fn is_eof(&mut self) -> bool {
        self.lexer.peek().get().is_none()
    }

    fn next_token(&mut self) -> Option<Token> {
        self.lexer.next()
    }

    pub fn command(&mut self) -> Option<Command> {
        if self.is_eof() {
            return None;
        }

        let token = self.next_token().unwrap();

        match token {
            Token::Lf => self.command(),
            Token::Int(tok) => match tok {
                IntToken::Deg => {
                    self.add_error("unexpected \"deg\", ignoring");
                    self.command()
                }
                IntToken::In => {
                    self.add_error("unexpected \"in\", ignoring");
                    self.command()
                }
                IntToken::Rad => {
                    self.add_error("unexpected \"rad\", ignoring");
                    self.command()
                }
                IntToken::Cm => {
                    self.add_error("unexpected \"cm\", ignoring");
                    self.command()
                }
                IntToken::ArmBackDown => {
                    if let Some(n) = self.deg() {
                        Some(Command::BackArmDown(n))
                    } else {
                        self.add_error("expected number after \"back arm down\". ignoring");
                        self.command()
                    }
                }
                IntToken::MoveBackward => {
                    if let Some(n) = self.cm() {
                        Some(Command::MoveBackward(n))
                    } else {
                        self.add_error("expected number after \"move backwards\". ignoring");
                        self.command()
                    }
                }
                IntToken::ArmBackUp => {
                    if let Some(n) = self.deg() {
                        Some(Command::BackArmUp(n))
                    } else {
                        self.add_error("expected number after \"back arm up\". ignoring");
                        self.command()
                    }
                }
                IntToken::ArmFrontDown => {
                    if let Some(n) = self.deg() {
                        Some(Command::FrontArmDown(n))
                    } else {
                        self.add_error("expected number after \"front arm down\". ignoring");
                        self.command()
                    }
                }
                IntToken::ArmFrontUp => {
                    if let Some(n) = self.deg() {
                        Some(Command::FrontArmUp(n))
                    } else {
                        self.add_error("expected number after \"front arm up\". ignoring");
                        self.command()
                    }
                }
                IntToken::Debug => Some(Command::PyDebug),
                IntToken::RotateLeft => {
                    if let Some(n) = self.deg() {
                        Some(Command::RotateLeft(n))
                    } else {
                        self.add_error("expected number after \"rotate left\". ignoring");
                        self.command()
                    }
                }
                IntToken::RotateRight => {
                    if let Some(n) = self.deg() {
                        Some(Command::RotateRight(n))
                    } else {
                        self.add_error("expected number after \"rotate right\". ignoring");
                        self.command()
                    }
                }
                IntToken::MoveForward => {
                    if let Some(n) = self.cm() {
                        Some(Command::MoveForward(n))
                    } else {
                        self.add_error("expected number after \"move forward\". ignoring");
                        self.command()
                    }
                }
                IntToken::Then => unreachable!(),
            },
            Token::Number(i) => {
                self.add_error(format!("unexpected number {i}, ignoring"));
                self.command()
            }
            Token::Errors(mut i) => {
                self.errors.append(&mut i);
                None
            }
        }
    }

    fn cm(&mut self) -> Option<f32> {
        if let Some((n, s)) = self.number() {
            let suffix = s.unwrap_or(NumberSuffix::Cm);
            let number = match suffix {
                NumberSuffix::Deg => {
                    self.add_error("cannot convert degrees to centimeters");
                    n
                }
                NumberSuffix::Rad => {
                    self.add_error("cannot convert radians to centimeters");
                    n
                }
                NumberSuffix::Cm => n,
                NumberSuffix::In => n * 2.54,
            };
            Some(number)
        } else {
            None
        }
    }

    fn deg(&mut self) -> Option<f32> {
        if let Some((n, s)) = self.number() {
            let suffix = s.unwrap_or(NumberSuffix::Deg);
            let number = match suffix {
                NumberSuffix::Cm => {
                    self.add_error("cannot convert centimeters to degrees");
                    n
                }
                NumberSuffix::In => {
                    self.add_error("cannot convert inches to degrees");
                    n
                }
                NumberSuffix::Deg => n,
                NumberSuffix::Rad => n * (180.0 / PI),
            };
            Some(number)
        } else {
            None
        }
    }

    fn number(&mut self) -> Option<(f32, Option<NumberSuffix>)> {
        if self.is_eof() {
            return None;
        }

        let token = self.next_token().unwrap();

        match token {
            Token::Lf => self.number(),
            Token::Number(i) => Some((i, self.number_suffix())),
            Token::Int(_) => None,
            Token::Errors(mut i) => {
                self.errors.append(&mut i);
                None
            }
        }
    }

    fn number_suffix(&mut self) -> Option<NumberSuffix> {
        if self.is_eof() {
            return None;
        }

        let peek = self.lexer.peek();
        let token = peek.get().unwrap();

        match token {
            Token::Lf => {
                self.next_token();
                self.number_suffix()
            }
            Token::Number(_) => None,
            Token::Int(int_token) => match int_token {
                IntToken::Cm => {
                    self.lexer.next();
                    Some(NumberSuffix::Cm)
                }
                IntToken::Rad => {
                    self.lexer.next();
                    Some(NumberSuffix::Rad)
                }
                IntToken::Deg => {
                    self.lexer.next();
                    Some(NumberSuffix::Deg)
                }
                IntToken::In => {
                    self.lexer.next();
                    Some(NumberSuffix::In)
                }
                _ => None,
            },
            Token::Errors(_) => {
                if let Some(Token::Errors(mut i)) = self.lexer.next() {
                    self.errors.append(&mut i);
                    None
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn finish(self) -> Vec<String> {
        self.errors
    }
}

enum NumberSuffix {
    Cm,
    In,
    Deg,
    Rad,
}
