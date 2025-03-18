use std::iter::FusedIterator;
use std::str::Chars;

/// Iterator over identifiers in a C++ source file.
#[derive(Clone)]
pub struct Identifiers<'a>(&'a str);

impl<'a> Identifiers<'a> {
    pub fn new(text: &'a str) -> Self {
        Identifiers(text)
    }
}

fn skip_string(chars: &mut Chars, delim: char) {
    loop {
        match chars.next() {
            None => return,
            Some(c) => {
                if c == delim {
                    return;
                }
                if c == '\\' {
                    chars.next();
                }
            }
        }
    }
}

/// Skip a "pp-number", after the leading digit or period and digit have been consumed.
fn skip_number(chars: &mut Chars) {
    loop {
        let saved = chars.clone();
        match chars.next() {
            None => return,
            Some(ch) => match ch {
                'e' | 'E' | 'p' | 'P' => {
                    let mut temp = chars.clone();
                    match temp.next() {
                        Some('-' | '+') => *chars = temp,
                        _ => (),
                    }
                }
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '.' => (),
                '\'' => match chars.next() {
                    Some('0'..='9' | 'a'..='z' | 'A'..='Z' | '_') => (),
                    _ => {
                        *chars = saved;
                        return;
                    }
                },
                _ => {
                    *chars = saved;
                    return;
                }
            },
        }
    }
}

impl<'a> Iterator for Identifiers<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.0.chars();
        'outer: loop {
            let saved = chars.clone();
            let ch = match chars.next() {
                None => break 'outer,
                Some(ch) => ch,
            };
            match ch {
                // String or character constant.
                '"' | '\'' => skip_string(&mut chars, ch),
                // Comment.
                '/' => match chars.next() {
                    None => break,
                    Some('/') => loop {
                        match chars.next() {
                            None => break 'outer,
                            Some('\n') | Some('\r') => break,
                            Some(_) => (),
                        }
                    },
                    Some('*') => loop {
                        match chars.next() {
                            None => break 'outer,
                            Some('*') => match chars.next() {
                                None => break 'outer,
                                Some('/') => break,
                                Some(_) => (),
                            },
                            Some(_) => (),
                        }
                    },
                    Some(_) => (),
                },
                // Number.
                '0'..='9' => skip_number(&mut chars),
                '.' => {
                    let mut temp = chars.clone();
                    if let Some(ch) = temp.next() {
                        if ch.is_ascii_digit() {
                            chars = temp;
                            skip_number(&mut chars);
                        }
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    loop {
                        let saved2 = chars.clone();
                        match chars.next() {
                            None => break,
                            Some('0'..='9' | 'a'..='z' | 'A'..='Z' | '_') => (),
                            _ => {
                                chars = saved2;
                                break;
                            }
                        }
                    }
                    let text = saved.as_str();
                    let rest = chars.as_str();
                    let text = &text[..text.len() - rest.len()];
                    self.0 = rest;
                    return Some(text);
                }
                _ => (),
            }
        }
        self.0 = "";
        None
    }
}

impl<'a> FusedIterator for Identifiers<'a> {}
