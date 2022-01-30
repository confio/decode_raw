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
}
