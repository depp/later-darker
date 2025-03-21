use core::fmt;
use std::error;
use std::str;

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

struct Evaluator<'a> {
    atoms: &'a dyn Atoms,
    text: &'a [u8],
    tok: Tok,
    value: &'a str,
}

impl<'a> Evaluator<'a> {
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

    fn eval_or(&mut self) -> Result<bool, Error> {
        let mut value = self.eval_and()?;
        while self.tok == Tok::Or {
            self.next_token();
            let rhs = self.eval_and()?;
            value = value || rhs;
        }
        Ok(value)
    }

    fn eval_and(&mut self) -> Result<bool, Error> {
        let mut value = self.eval_not()?;
        while self.tok == Tok::And {
            self.next_token();
            let rhs = self.eval_not()?;
            value = value && rhs;
        }
        Ok(value)
    }

    fn eval_not(&mut self) -> Result<bool, Error> {
        let mut flip = false;
        while self.tok == Tok::Not {
            self.next_token();
            flip = !flip;
        }
        let value = self.eval_atom()?;
        Ok(if flip { !value } else { value })
    }

    fn eval_atom(&mut self) -> Result<bool, Error> {
        match self.tok {
            Tok::Atom => match self.atoms.evaluate(self.value) {
                None => Err(Error::UnknownAtom(self.value.to_string())),
                Some(value) => {
                    self.next_token();
                    Ok(value)
                }
            },
            Tok::Open => {
                self.next_token();
                let expr = self.eval_or()?;
                if self.tok != Tok::Close {
                    return Err(Error::InvalidSyntax);
                }
                self.next_token();
                Ok(expr)
            }
            _ => Err(Error::InvalidSyntax),
        }
    }
}

/// Error when evaluating a build tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidToken,
    InvalidSyntax,
    UnknownAtom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidToken => f.write_str("invalid token"),
            Error::InvalidSyntax => f.write_str("invalid syntax"),
            Error::UnknownAtom(name) => write!(f, "unknown identifier in build tag: {}", name),
        }
    }
}

impl error::Error for Error {}

pub fn evaluate(atoms: &dyn Atoms, text: &[u8]) -> Result<bool, Error> {
    let mut evaluator = Evaluator {
        text,
        tok: Tok::End,
        value: "",
        atoms,
    };
    evaluator.next_token();
    let value = evaluator.eval_or();
    if evaluator.tok == Tok::Error {
        return Err(Error::InvalidToken);
    }
    let value = value?;
    if evaluator.tok != Tok::End {
        return Err(Error::InvalidSyntax);
    }
    Ok(value)
}

/// Build token atom lookup.
pub trait Atoms {
    /// Evaluate an atom, or return None if the atom does not exist.
    fn evaluate(&self, atom: &str) -> Option<bool>;
}

#[cfg(test)]
mod test {
    use super::{Atoms, Error, evaluate};

    struct SimpleAtoms([bool; 3]);

    impl Atoms for SimpleAtoms {
        fn evaluate(&self, atom: &str) -> Option<bool> {
            Some(match atom {
                "true" => true,
                "false" => false,
                "x" => self.0[0],
                "y" => self.0[1],
                "z" => self.0[2],
                _ => return None,
            })
        }
    }

    fn check(text: &str, result: bool) {
        let value =
            evaluate(&SimpleAtoms([false; 3]), text.as_bytes()).expect("Evaluation should succeed");
        assert_eq!(result, value);
    }

    fn check_matrix(text: &str, f: fn(x: bool, y: bool, z: bool) -> bool) {
        for n in 0..8 {
            let x = n & 4 != 0;
            let y = n & 2 != 0;
            let z = n & 1 != 0;
            let value = evaluate(&SimpleAtoms([x, y, z]), text.as_bytes())
                .expect("Evaluation should succeed");
            let expect = f(x, y, z);
            assert_eq!(value, expect);
        }
    }

    fn check_err(text: &str, err: Error) {
        let value = evaluate(&SimpleAtoms([false; 3]), text.as_bytes());
        assert_eq!(value, Err(err) as Result<bool, Error>);
    }

    #[test]
    fn test_atom() {
        check("true", true);
        check("false", false);
        check("  true  ", true);
        check_err("unknown", Error::UnknownAtom("unknown".to_string()));
    }

    #[test]
    fn test_and() {
        check_matrix("x && y", |x, y, _| x && y);
        check_matrix("x&&y&&z", |x, y, z| x && y && z);
    }

    #[test]
    fn test_or() {
        check_matrix("x || y", |x, y, _| x || y);
        check_matrix("x||y||z", |x, y, z| x || y || z);
    }

    #[test]
    fn test_not() {
        check_matrix("!x", |x, _, _| !x);
        check_matrix("!!x", |x, _, _| x);
    }

    #[test]
    fn test_precedence() {
        check_matrix("!x && y || !z", |x, y, z| !x && y || !z);
        check_matrix("!x && (y || !z)", |x, y, z| !x && (y || !z));
    }
}
