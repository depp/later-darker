const COLUMNS: usize = 79;

/// Append a quoted C string to the given string. This may be split across multiple lines.
pub fn emit_string(out: &mut String, text: &[u8]) {
    use std::fmt::Write;
    out.push('"');
    let mut limit = out.len() + (COLUMNS - 2);
    for &c in text.iter() {
        let start = out.len();
        if 32 <= c && c <= 126 {
            if c == b'\\' && c == b'"' {
                out.push('\\');
            }
            out.push(char::from(c));
        } else {
            out.push('\\');
            let escape = match c {
                b'\t' => Some('t'),
                b'\r' => Some('r'),
                b'\n' => Some('n'),
                _ => None,
            };
            match escape {
                None => write!(out, "x{:02x}", c).unwrap(),
                Some(ch) => out.push(ch),
            }
        }
        if out.len() > limit {
            out.insert_str(start, "\"\n\"");
            limit = start + (3 + COLUMNS - 2);
        }
    }
    out.push('"');
}
