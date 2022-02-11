use protofish::decode::UnknownValue;
use protofish::prelude::*;
use protofish::prelude::{Context, FieldValue};

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub path: Vec<u64>,
    pub value: EntryValue,
}

#[derive(Debug, PartialEq)]
pub enum EntryValue {
    /// Wire type 1 (64-bit value) used for fixed64, sfixed64, double
    Fixed64(u64),
    // Wire type 5 (32-bit value) used for fixed32, sfixed32, float
    Fixed32(u32),
    /// Wire type 0 (Varint) used for int32, int64, uint32, uint64, sint32, sint64, bool, enum
    Varint(u128),
    /// Wire type 2 (length delimited).
    Bytes(Vec<u8>),
    OpenNested,
    CloseNested,
}

#[derive(Copy, Clone)]
pub struct ParseConfig {
    pub no_fixed64: bool,
    pub no_fixed32: bool,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self {
            no_fixed64: false,
            no_fixed32: false,
        }
    }
}

/// Tries to parse bytes as protobuf message and returns entries.
/// Each entry represents one line in the output.
pub fn try_parse_entries(bytes: &[u8], config: ParseConfig) -> Option<Vec<Entry>> {
    try_parse_entries_inner(bytes, config, &[])
}

/// The implementation for try_parse_entries.
///
/// The extra path argument is the position in the larger structure
/// where the currently expected bytes were found. This is required
/// to be able return the absolute path in the resulting entry.
fn try_parse_entries_inner(bytes: &[u8], config: ParseConfig, path: &[u64]) -> Option<Vec<Entry>> {
    if bytes.is_empty() {
        // Empty byte arrays should be represented as "" instead of empty message
        return None;
    }

    let fields = decode_fields(&bytes);
    let mut out = Vec::<Entry>::new();
    for field in fields.into_iter() {
        let mut nested_path = path.to_vec();
        nested_path.push(field.number);

        match &field.value {
            Value::Unknown(unknown) => match unknown {
                UnknownValue::Fixed64(v) => {
                    if config.no_fixed64 {
                        return None;
                    }
                    out.push(Entry {
                        path: nested_path,
                        value: EntryValue::Fixed64(*v),
                    })
                }
                UnknownValue::Fixed32(v) => {
                    if config.no_fixed32 {
                        return None;
                    }
                    out.push(Entry {
                        path: nested_path,
                        value: EntryValue::Fixed32(*v),
                    })
                }
                UnknownValue::Varint(v) => out.push(Entry {
                    path: nested_path,
                    value: EntryValue::Varint(*v),
                }),
                UnknownValue::VariableLength(v) => {
                    if let Some(nested_entries) = try_parse_entries_inner(v, config, &nested_path) {
                        out.push(Entry {
                            path: nested_path.clone(),
                            value: EntryValue::OpenNested,
                        });
                        out.extend(nested_entries);
                        out.push(Entry {
                            path: nested_path,
                            value: EntryValue::CloseNested,
                        });
                    } else {
                        out.push(Entry {
                            path: nested_path,
                            value: EntryValue::Bytes(v.to_vec()),
                        })
                    }
                }
                UnknownValue::Invalid(_wire_type, _bytes) => {
                    return None;
                }
            },
            _ => return None,
        };
    }
    Some(out)
}

pub fn decode_fields(bytes: &[u8]) -> Vec<FieldValue> {
    let context = Context::parse(&[r#"
        syntax = "proto3";
        package Proto;

        message Empty { }
    "#])
    .unwrap();

    let request = context.get_message("Proto.Empty").unwrap();
    let value = request.decode(bytes, &context);
    value.fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_parse_entries_works() {
        // one
        let entries = try_parse_entries(b"\x12\x07Unknown", ParseConfig::default()).unwrap();
        assert_eq!(
            entries,
            &[Entry {
                path: vec![2],
                value: EntryValue::Bytes(b"Unknown".to_vec())
            }]
        );

        // two
        let entries =
            try_parse_entries(b"\x12\x07Unknown\x12\x07Unknown", ParseConfig::default()).unwrap();
        assert_eq!(
            entries,
            &[
                Entry {
                    path: vec![2],
                    value: EntryValue::Bytes(b"Unknown".to_vec())
                },
                Entry {
                    path: vec![2],
                    value: EntryValue::Bytes(b"Unknown".to_vec())
                }
            ]
        );

        // No valid protobuf (incomplete)
        let res = try_parse_entries(b"\x12\x07Unknown\x0a\x0fAtlantic ", ParseConfig::default());
        assert_eq!(res, None);

        // No valid protobuf (wrong wire type)
        // End group (deprecated) in field 2: hex((2 << 3) | 4)
        let res = try_parse_entries(b"\x14\x07Unknown", ParseConfig::default());
        assert_eq!(res, None);
    }

    #[test]
    fn try_parse_entries_returns_none_for_empty() {
        let res = try_parse_entries(b"", ParseConfig::default());
        assert_eq!(res, None);
    }

    #[test]
    fn try_parse_entries_inner_works() {
        // one
        let entries =
            try_parse_entries_inner(b"\x12\x07Unknown", ParseConfig::default(), &[]).unwrap();
        assert_eq!(
            entries,
            &[Entry {
                path: vec![2],
                value: EntryValue::Bytes(b"Unknown".to_vec())
            }]
        );

        // two
        let entries = try_parse_entries_inner(
            b"\x12\x07Unknown\x12\x07Unknown",
            ParseConfig::default(),
            &[],
        )
        .unwrap();
        assert_eq!(
            entries,
            &[
                Entry {
                    path: vec![2],
                    value: EntryValue::Bytes(b"Unknown".to_vec())
                },
                Entry {
                    path: vec![2],
                    value: EntryValue::Bytes(b"Unknown".to_vec())
                }
            ]
        );

        // nested path
        let entries =
            try_parse_entries_inner(b"\x12\x07Unknown", ParseConfig::default(), &[42]).unwrap();
        assert_eq!(
            entries,
            &[Entry {
                path: vec![42, 2],
                value: EntryValue::Bytes(b"Unknown".to_vec())
            }]
        );

        // No valid protobuf (incomplete)
        let res = try_parse_entries_inner(
            b"\x12\x07Unknown\x0a\x0fAtlantic ",
            ParseConfig::default(),
            &[],
        );
        assert_eq!(res, None);

        // No valid protobuf (wrong wire type)
        // End group (deprecated) in field 2: hex((2 << 3) | 4)
        let res = try_parse_entries_inner(b"\x14\x07Unknown", ParseConfig::default(), &[]);
        assert_eq!(res, None);
    }
}
