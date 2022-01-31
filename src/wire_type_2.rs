//! Everything about wire type 2 (Length-delimited),
//! i.e. string, bytes, embedded messages, packed repeated fields.

pub fn escape_string(input: &str) -> String {
    let escaped = snailquote::escape(input);
    if !escaped.starts_with('"') && !escaped.starts_with('\'') {
        format!("\"{}\"", escaped)
    } else {
        escaped.to_string()
    }
}

#[derive(Debug, PartialEq)]
pub enum ShowAs<'a> {
    String(&'a str),
    Bytes(&'a [u8]),
}

pub fn show_as<'a>(bytes: &'a [u8]) -> ShowAs<'a> {
    match std::str::from_utf8(bytes) {
        Ok(converted) => {
            if converted.chars().all(|char| match char {
                '\t' | '\r' | '\n' => true, // Part of next range but should be allowed
                '\0'..='\x19' => false,     // Non-printable ASCII characters
                _ => true,
            }) {
                ShowAs::String(converted)
            } else {
                ShowAs::Bytes(bytes)
            }
        }
        Err(_err) => ShowAs::Bytes(bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_string_works() {
        assert_eq!(escape_string(""), r#""""#);
        assert_eq!(escape_string("a"), r#""a""#);
        assert_eq!(escape_string("foo"), r#""foo""#);

        // Spaces
        assert_eq!(escape_string("foo bar"), r#"'foo bar'"#);

        // Uses single quotes if that avoids escaping
        assert_eq!(escape_string("fo\"o"), r#"'fo"o'"#);
        assert_eq!(escape_string("{\"my\":\"json\"}"), r#"'{"my":"json"}'"#);

        // Uses double quotes if both single and double are in content
        assert_eq!(escape_string("f'o\"o"), r#""f'o\"o""#);
        // This case would use single quotes in prettier which counts single and double
        assert_eq!(
            escape_string("{\"my\":\"json's\"}"),
            r#""{\"my\":\"json's\"}""#
        );
    }

    #[test]
    fn show_as_works() {
        assert_eq!(show_as(b""), ShowAs::String(""));
        assert_eq!(show_as(b"123"), ShowAs::String("123"));
        assert_eq!(show_as(b"with space"), ShowAs::String("with space"));
        assert_eq!(show_as(b"Newline: \n"), ShowAs::String("Newline: \n"));
        assert_eq!(show_as(b"Tab: \t"), ShowAs::String("Tab: \t"));
        assert_eq!(show_as(b"CR: \r"), ShowAs::String("CR: \r"));

        // Invalid UTF8
        let non_utf8 = vec![0, 159, 146, 150];
        assert_eq!(show_as(&non_utf8), ShowAs::Bytes(&non_utf8));

        // Non-printable ASCII characters are valid UTF8 but should not be printed as string
        assert_eq!(show_as(b"__\0__"), ShowAs::Bytes(b"__\0__")); // Null
        assert_eq!(show_as(b"__\x07__"), ShowAs::Bytes(b"__\x07__")); // Bell
        assert_eq!(show_as(b"__\x0b__"), ShowAs::Bytes(b"__\x0b__")); // Vertical Tab
    }
}
