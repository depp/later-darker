use arcstr::ArcStr;
use core::fmt;
use std::error;
use std::str;
use std::sync::Arc;

// ============================================================================
// Errors
// ============================================================================

/// Error when parsing a build condition.
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

/// Error when evaluating a build condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError(pub ArcStr);

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "undefined identifier: {}", self.0)
    }
}

impl error::Error for EvalError {}

// ============================================================================
// Condition
// ============================================================================

/// A build condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Value(bool),
    Atom(ArcStr),
    Operation(Arc<Operation>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Not(Condition),
    And(Condition, Condition),
    Or(Condition, Condition),
}

impl ToString for Condition {
    fn to_string(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out
    }
}

/// Helper for writing binary conditions.
fn write_binary(
    lhs: &Condition,
    rhs: &Condition,
    out: &mut String,
    prec: i32,
    op_prec: i32,
    symbol: &str,
) {
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

impl Condition {
    /// Return the logical conjunction of two expressions.
    pub fn and(&self, other: &Self) -> Self {
        match self {
            Condition::Value(value) => {
                if *value {
                    other.clone()
                } else {
                    Condition::Value(false)
                }
            }
            _ => match other {
                Condition::Value(value) => {
                    if *value {
                        self.clone()
                    } else {
                        Condition::Value(false)
                    }
                }
                _ => Condition::Operation(Arc::new(Operation::And(self.clone(), other.clone()))),
            },
        }
    }

    /// Parse a build condition.
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
        Ok(expr)
    }

    /// Evaluate the condition.
    pub fn evaluate<F>(&self, eval_atom: F) -> Result<bool, EvalError>
    where
        F: Fn(&str) -> Option<bool>,
    {
        self.evaluate_impl(&eval_atom)
    }

    /// Write an condition in the given precedence context. The initial context
    /// is 0, and higher contexts bind more tightly.
    fn write(&self, out: &mut String, prec: i32) {
        match self {
            Condition::Value(value) => out.push_str(if *value { "true" } else { "false" }),
            Condition::Atom(atom) => out.push_str(atom),
            Condition::Operation(op) => op.write(out, prec),
        }
    }

    pub fn evaluate_impl<F>(&self, eval_atom: &F) -> Result<bool, EvalError>
    where
        F: Fn(&str) -> Option<bool>,
    {
        Ok(match self {
            Condition::Value(value) => *value,
            Condition::Atom(atom) => match eval_atom(atom) {
                None => return Err(EvalError(atom.clone())),
                Some(value) => value,
            },
            Condition::Operation(op) => return op.evaluate_impl(eval_atom),
        })
    }
}

impl Operation {
    fn write(&self, out: &mut String, prec: i32) {
        match self {
            Operation::Not(expr) => {
                out.push('!');
                expr.write(out, 2);
            }
            Operation::And(lhs, rhs) => write_binary(lhs, rhs, out, prec, 1, "&&"),
            Operation::Or(lhs, rhs) => write_binary(lhs, rhs, out, prec, 0, "||"),
        }
    }

    fn evaluate_impl<F>(&self, eval_atom: &F) -> Result<bool, EvalError>
    where
        F: Fn(&str) -> Option<bool>,
    {
        Ok(match self {
            Operation::Not(expr) => !expr.evaluate_impl(eval_atom)?,
            Operation::And(lhs, rhs) => {
                let lhs = lhs.evaluate_impl(eval_atom)?;
                let rhs = rhs.evaluate_impl(eval_atom)?;
                lhs && rhs
            }
            Operation::Or(lhs, rhs) => {
                let lhs = lhs.evaluate_impl(eval_atom)?;
                let rhs = rhs.evaluate_impl(eval_atom)?;
                lhs || rhs
            }
        })
    }

    fn condition(self) -> Condition {
        Condition::Operation(Arc::new(self))
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

    fn parse_or(&mut self) -> Result<Condition, ParseError> {
        let mut expr = self.parse_and()?;
        while self.tok == Tok::Or {
            self.next_token();
            let rhs = self.parse_and()?;
            expr = Operation::Or(expr, rhs).condition();
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Condition, ParseError> {
        let mut expr = self.parse_not()?;
        while self.tok == Tok::And {
            self.next_token();
            let rhs = self.parse_not()?;
            expr = Operation::And(expr, rhs).condition();
        }
        Ok(expr)
    }

    fn parse_not(&mut self) -> Result<Condition, ParseError> {
        let mut flip = false;
        while self.tok == Tok::Not {
            self.next_token();
            flip = !flip;
        }
        let expr = self.parse_atom()?;
        Ok(if flip {
            Operation::Not(expr).condition()
        } else {
            expr
        })
    }

    fn parse_atom(&mut self) -> Result<Condition, ParseError> {
        match self.tok {
            Tok::Atom => {
                let expr = match self.value {
                    "false" => Condition::Value(false),
                    "true" => Condition::Value(true),
                    _ => Condition::Atom(ArcStr::from(self.value)),
                };
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
    use super::{Condition, Operation, ParseError};
    use arcstr::ArcStr;
    use std::sync::Arc;

    fn check_parse(text: &str, expected: Condition) {
        let result = Condition::parse(text.as_bytes()).expect("Parsing should succeed.");
        assert_eq!(result, expected);
    }

    fn check_err(text: &str, err: ParseError) {
        let result = Condition::parse(text.as_bytes()).expect_err("Parsing should fail.");
        assert_eq!(result, err);
    }

    fn var(x: &'static str) -> Condition {
        Condition::Atom(ArcStr::from(x))
    }

    fn enot(x: Condition) -> Condition {
        Condition::Operation(Arc::new(Operation::Not(x)))
    }

    fn eor(x: Condition, y: Condition) -> Condition {
        Condition::Operation(Arc::new(Operation::Or(x, y)))
    }

    fn eand(x: Condition, y: Condition) -> Condition {
        Condition::Operation(Arc::new(Operation::And(x, y)))
    }

    #[test]
    fn test_parse_atom() {
        check_parse("value", var("value"));
        check_parse("  atom  ", var("atom"));
        check_parse("true", Condition::Value(true));
        check_parse("false", Condition::Value(false));
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
