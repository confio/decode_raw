use std::ops::Deref;

use crate::parse::Entry;

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
        // Trim leading .
        let prepared = if input.starts_with('.') {
            &input[1..]
        } else {
            input
        };
        if prepared.is_empty() {
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

/// Check if the given entry is selected by the select query.
///
/// Right now this means the query must be a prefix of the path.
/// But you could imagine more complex queries as well later on.
pub fn is_selected(entry: &Entry, query: &SelectQuery) -> bool {
    entry.path.starts_with(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntryValue;

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

        // Empty components
        let err = SelectQuery::parse(".1..2").unwrap_err();
        assert_eq!(err, "cannot parse integer from empty string");
        let err = SelectQuery::parse(".1.").unwrap_err();
        assert_eq!(err, "cannot parse integer from empty string");
        let err = SelectQuery::parse("..").unwrap_err();
        assert_eq!(err, "cannot parse integer from empty string");
        let err = SelectQuery::parse("..1").unwrap_err();
        assert_eq!(err, "cannot parse integer from empty string");
    }

    #[test]
    fn is_selected_works() {
        let entry = Entry {
            path: vec![1, 2, 3],
            value: EntryValue::Int(1),
        };
        assert!(is_selected(&entry, &SelectQuery::parse(".1").unwrap()));
        assert!(is_selected(&entry, &SelectQuery::parse(".1.2").unwrap()));
        assert!(is_selected(&entry, &SelectQuery::parse(".1.2.3").unwrap()));
        assert!(!is_selected(
            &entry,
            &SelectQuery::parse(".1.2.3.4").unwrap()
        ));
        assert!(!is_selected(&entry, &SelectQuery::parse(".5").unwrap()));
    }
}
