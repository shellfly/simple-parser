use std::fmt;

use regex::Regex;

// Token is either a Symbol like "1", "2", or an Op like "+", "*"
#[derive(Debug, Clone)]
pub enum Token {
    Symbol(String),
    Op(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Symbol(s) => write!(f, "{}", s),
            Token::Op(op) => write!(f, "{}", op),
        }
    }
}

impl Token {
    fn is_left_paren(&self) -> bool {
        match self {
            Self::Op(op) => op == "(",
            _ => false,
        }
    }
    fn is_right_paren(&self) -> bool {
        match self {
            Self::Op(op) => op == ")",
            _ => false,
        }
    }
    // nud return right binding power
    fn nud(&self) -> Option<u8> {
        match self {
            Token::Op(s) => match s.as_str() {
                "+" | "-" => Some(30),
                _ => None,
            },
            _ => None,
        }
    }

    // led return left and right binding power
    fn led(&self) -> Option<(u8, u8)> {
        match self {
            Token::Op(s) => match s.as_str() {
                "+" | "-" => Some((10, 10)),
                "*" | "/" => Some((20, 20)),
                _ => None,
            },
            _ => None,
        }
    }
}

struct Lexer {
    tokens: Vec<Token>,
}
impl Lexer {
    fn new(input: &str) -> Lexer {
        let re = Regex::new(r"([-+*/()])").unwrap();
        let mut tokens = re
            .replace_all(input, r" ${1} ")
            .split_whitespace()
            .map(|c| match c {
                "+" | "-" | "*" | "/" | "(" | ")" => Token::Op(c.to_string()),
                _ => Token::Symbol(c.to_string()),
            })
            .collect::<Vec<_>>();
        // parse tokens from left to right, reverse tokens here so that we can
        // pop out first token without shifting all elements.
        tokens.reverse();
        Lexer { tokens }
    }

    // pop the first token of origin input, from left to right
    fn pop(&mut self) -> Option<Token> {
        self.tokens.pop()
    }
    // check first token without pop out it
    fn peek(&mut self) -> Option<&Token> {
        self.tokens.last()
    }
}

// Expr is a lisp S-expression, it's either an atom or a list of atom.
pub enum Expr {
    Atom(Token),
    Cons(Token, Vec<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Atom(t) => write!(f, "{}", t),
            Expr::Cons(head, rest) => {
                write!(f, "({}", head)?;
                for s in rest {
                    write!(f, " {}", s)?
                }
                write!(f, ")")
            }
        }
    }
}

// parse receive an input text and transform it to Expr
pub fn parse(input: &str) -> Expr {
    let mut lexer = Lexer::new(input);
    parse_bp(&mut lexer, 0)
}

fn parse_bp(lexer: &mut Lexer, rbp: u8) -> Expr {
    let token = lexer.pop().expect("unexpected eof");
    let mut left = match token {
        Token::Symbol(_) => Expr::Atom(token),
        Token::Op(_) => {
            if token.is_left_paren() {
                let left = parse_bp(lexer, 0);
                lexer.pop().expect("unmatch parentheses");
                left
            } else {
                let new_rbp = token
                    .nud()
                    .unwrap_or_else(|| panic!("unexpected prefix op: {:?}", token));
                let right = parse_bp(lexer, new_rbp);
                Expr::Cons(token, vec![right])
            }
        }
    };

    while let Some(token) = lexer.peek() {
        if token.is_right_paren() {
            break;
        }

        let token = token.clone();
        let (lbp, new_rbp) = token
            .led()
            .unwrap_or_else(|| panic!("unexpected token: {:?}", token));
        if lbp < rbp {
            break;
        }
        lexer.pop(); // pop out operator
        let right = parse_bp(lexer, new_rbp);
        left = Expr::Cons(token, vec![left, right])
    }
    left
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infix() {
        let s = parse("1 + 2 * 3");
        assert_eq!(s.to_string(), "(+ 1 (* 2 3))");

        let s = parse("1 * 2 + 3");
        assert_eq!(s.to_string(), "(+ (* 1 2) 3)");
    }

    #[test]
    fn test_prefix() {
        let s = parse("-1*2 + 3");
        assert_eq!(s.to_string(), "(+ (* (- 1) 2) 3)");
    }

    #[test]
    fn test_group() {
        let s = parse("(-1+2) * 3");
        assert_eq!(s.to_string(), "(* (+ (- 1) 2) 3)");

        let s = parse("( -1 + 2 ) * 3 - -4");
        assert_eq!(s.to_string(), "(- (* (+ (- 1) 2) 3) (- 4))");
    }
}
