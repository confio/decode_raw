use std::ops::Deref;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct SelectQuery(Vec<u64>);

impl Deref for SelectQuery {
    type Target = [u64];

    fn deref(&self) -> &[u64] {
        &self.0
    }
}

impl SelectQuery {
    pub fn parse(input: &str) -> Result<Self, String> {
        let prepared = input.trim_start_matches('.');
        if prepared.len() == 0 {
            return Ok(SelectQuery::default());
        }
        let components: Vec<&str> = prepared.split('.').collect();
        let path: Vec<u64> = components
            .into_iter()
            .map(|str| str.parse::<u64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;
        Ok(SelectQuery(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_can_parse_empty() {
        assert_eq!(SelectQuery::parse("").unwrap().0, &[]);
        assert_eq!(SelectQuery::parse(".").unwrap().0, &[]);
    }

    #[test]
    fn parse_can_parse_simple() {
        assert_eq!(SelectQuery::parse("1").unwrap().0, &[1]);
        assert_eq!(SelectQuery::parse(".1").unwrap().0, &[1]);
    }

    #[test]
    fn parse_can_parse_multi() {
        assert_eq!(SelectQuery::parse("1.2").unwrap().0, &[1, 2]);
        assert_eq!(SelectQuery::parse(".1.2").unwrap().0, &[1, 2]);
        assert_eq!(SelectQuery::parse("3.3").unwrap().0, &[3, 3]);
    }

    #[test]
    fn parse_handles_error() {
        let err = SelectQuery::parse("1.2_3").unwrap_err();
        assert_eq!(err, "invalid digit found in string");
        let err = SelectQuery::parse("1.2 3").unwrap_err();
        assert_eq!(err, "invalid digit found in string");

        let err = SelectQuery::parse("2_3").unwrap_err();
        assert_eq!(err, "invalid digit found in string");
        let err = SelectQuery::parse("2 3").unwrap_err();
        assert_eq!(err, "invalid digit found in string");
    }
}
