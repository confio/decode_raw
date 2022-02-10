pub fn parse_select(input: &str) -> Vec<u64> {
    let prepared = input.trim_start_matches('.');
    if prepared.len() == 0 {
        return Vec::default();
    }
    let components: Vec<&str> = prepared.split('.').collect();
    let path: Vec<u64> = components
        .into_iter()
        .map(|str| str.parse::<u64>().unwrap())
        .collect();
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_empty() {
        assert_eq!(parse_select(""), &[]);
        assert_eq!(parse_select("."), &[]);
    }

    #[test]
    fn can_parse_simple() {
        assert_eq!(parse_select("1"), &[1]);
        assert_eq!(parse_select(".1"), &[1]);
    }

    #[test]
    fn can_parse_multi() {
        assert_eq!(parse_select("1.2"), &[1, 2]);
        assert_eq!(parse_select(".1.2"), &[1, 2]);
        assert_eq!(parse_select("3.3"), &[3, 3]);
    }
}
