use arcstr::ArcStr;
use core::fmt;
use std::error;
use std::str;

// ============================================================================
// Errors
// ============================================================================

/// Error when parsing a build expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidToken,
    InvalidSyntax,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            ParseError::InvalidToken => "invalid token",
            ParseError::InvalidSyntax => "invalid syntax",
        })
    }
}

impl error::Error for ParseError {}

/// Error when evaluating a build expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError(pub ArcStr);

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "undefined identifier: {}", self.0)
    }
}

impl error::Error for EvalError {}

// ============================================================================
// Expression
// ============================================================================

/// A build tag expression.
#[derive(Debug)]
pub struct Expression(Expr);

impl Expression {
    /// Parse a build expression.
    pub fn parse(text: &[u8]) -> Result<Self, ParseError> {
        let mut parser = Parser {
            text,
            tok: Tok::End,
            value: "",
        };
        parser.next_token();
        let value = parser.parse_or();
        if parser.tok == Tok::Error {
            return Err(ParseError::InvalidToken);
        }
        let expr = value?;
        if parser.tok != Tok::End {
            return Err(ParseError::InvalidSyntax);
        }
        Ok(Expression(expr))
    }

    /// Evaluate the expression.
    pub fn evaluate<F>(&self, eval_atom: &F) -> Result<bool, EvalError>
    where
        F: Fn(&str) -> Option<bool>,
    {
        self.0.evaluate(eval_atom)
    }
}

impl ToString for Expression {
    fn to_string(&self) -> String {
        let mut out = String::new();
        self.0.write(&mut out, 0);
        out
    }
}

/// A build tag expression.
#[derive(Debug, PartialEq, Eq)]
enum Expr {
    Atom(ArcStr),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

/// Helper for writing binary expressions.
fn write_binary(lhs: &Expr, rhs: &Expr, out: &mut String, prec: i32, op_prec: i32, symbol: &str) {
    let group = prec > op_prec;
    if group {
        out.push('(');
    }
    lhs.write(out, op_prec);
    out.push(' ');
    out.push_str(symbol);
    out.push(' ');
    rhs.write(out, op_prec);
    if group {
        out.push(')');
    }
}

impl Expr {
    /// Write an expression in the given precedence context. The initial context
    /// is 0, and higher contexts bind more tightly.
    fn write(&self, out: &mut String, prec: i32) {
        match self {
            Expr::Atom(atom) => out.push_str(atom),
            Expr::Not(expr) => {
                out.push('!');
                expr.write(out, 2);
            }
            Expr::And(lhs, rhs) => write_binary(lhs, rhs, out, prec, 1, "&&"),
            Expr::Or(lhs, rhs) => write_binary(lhs, rhs, out, prec, 0, "||"),
        }
    }

    pub fn evaluate<F>(&self, eval_atom: &F) -> Result<bool, EvalError>
    where
        F: Fn(&str) -> Option<bool>,
    {
        Ok(match self {
            Expr::Atom(atom) => match eval_atom(atom) {
                None => return Err(EvalError(atom.clone())),
                Some(value) => value,
            },
            Expr::Not(expr) => !expr.evaluate(eval_atom)?,
            Expr::And(lhs, rhs) => {
                let lhs = lhs.evaluate(eval_atom)?;
                let rhs = rhs.evaluate(eval_atom)?;
                lhs && rhs
            }
            Expr::Or(lhs, rhs) => {
                let lhs = lhs.evaluate(eval_atom)?;
                let rhs = rhs.evaluate(eval_atom)?;
                lhs || rhs
            }
        })
    }
}

// ============================================================================
// Parsing
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tok {
    End,
    Error,
    Atom,
    Not,
    Open,
    Close,
    And,
    Or,
}

struct Parser<'a> {
    text: &'a [u8],
    tok: Tok,
    value: &'a str,
}

impl<'a> Parser<'a> {
    fn next_token(&mut self) {
        let start = self.text.trim_ascii_start();
        self.tok = Tok::Error;
        self.value = "";
        let Some((&c, rest)) = start.split_first() else {
            self.tok = Tok::End;
            self.text = start;
            return;
        };
        let (tok, n) = match c {
            b'!' => (Tok::Not, 0),
            b'(' => (Tok::Open, 0),
            b')' => (Tok::Close, 0),
            b'&' => {
                if rest.starts_with(b"&") {
                    (Tok::And, 1)
                } else {
                    return;
                }
            }
            b'|' => {
                if rest.starts_with(b"|") {
                    (Tok::Or, 1)
                } else {
                    return;
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let n = rest
                    .iter()
                    .position(|&c| !c.is_ascii_alphanumeric() && c != b'_')
                    .unwrap_or(rest.len());
                self.value = str::from_utf8(&start[..n + 1]).unwrap();
                (Tok::Atom, n)
            }
            _ => return,
        };
        self.text = &rest[n..];
        self.tok = tok;
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_and()?;
        while self.tok == Tok::Or {
            self.next_token();
            let rhs = self.parse_and()?;
            expr = Expr::Or(Box::new(expr), Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_not()?;
        while self.tok == Tok::And {
            self.next_token();
            let rhs = self.parse_not()?;
            expr = Expr::And(Box::new(expr), Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        let mut flip = false;
        while self.tok == Tok::Not {
            self.next_token();
            flip = !flip;
        }
        let expr = self.parse_atom()?;
        Ok(if flip {
            Expr::Not(Box::new(expr))
        } else {
            expr
        })
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        match self.tok {
            Tok::Atom => {
                let expr = Expr::Atom(ArcStr::from(self.value));
                self.next_token();
                Ok(expr)
            }
            Tok::Open => {
                self.next_token();
                let expr = self.parse_or()?;
                if self.tok != Tok::Close {
                    return Err(ParseError::InvalidSyntax);
                }
                self.next_token();
                Ok(expr)
            }
            _ => Err(ParseError::InvalidSyntax),
        }
    }
}

#[cfg(test)]
mod test {
    use arcstr::ArcStr;

    use super::{Expr, Expression, ParseError};

    fn check_parse(text: &str, expected: Expr) {
        let result = Expression::parse(text.as_bytes()).expect("Parsing should succeed.");
        assert_eq!(result.0, expected);
    }

    fn check_err(text: &str, err: ParseError) {
        let result = Expression::parse(text.as_bytes()).expect_err("Parsing should fail.");
        assert_eq!(result, err);
    }

    fn var(x: &'static str) -> Expr {
        Expr::Atom(ArcStr::from(x))
    }

    fn enot(x: Expr) -> Expr {
        Expr::Not(Box::new(x))
    }

    fn eor(x: Expr, y: Expr) -> Expr {
        Expr::Or(Box::new(x), Box::new(y))
    }

    fn eand(x: Expr, y: Expr) -> Expr {
        Expr::And(Box::new(x), Box::new(y))
    }

    #[test]
    fn test_parse_atom() {
        check_parse("true", var("true"));
        check_parse("  false  ", var("false"));
    }

    #[test]
    fn test_parse_op() {
        check_parse("x && y", eand(var("x"), var("y")));
        check_parse("!x", enot(var("x")));
        check_parse("x && y || z", eor(eand(var("x"), var("y")), var("z")));
        check_parse("x || y && z", eor(var("x"), eand(var("y"), var("z"))));
        check_parse(
            "(x&&((z||!a)))",
            eand(var("x"), eor(var("z"), enot(var("a")))),
        );
    }

    #[test]
    fn test_fail() {
        check_err("", ParseError::InvalidSyntax);
        check_err("&&", ParseError::InvalidSyntax);
        check_err("(x", ParseError::InvalidSyntax);
        check_err("x && y ||", ParseError::InvalidSyntax);
        check_err("x y", ParseError::InvalidSyntax);
    }
}
