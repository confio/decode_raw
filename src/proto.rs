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
    Int(u128),
    Bytes(Vec<u8>),
    OpenNested,
    CloseNested,
}

/// Tries to parse bytes as protobuf message and returns entries.
/// Each entry represents one line in the output.
pub fn try_parse_entries(bytes: &[u8], path: &[u64]) -> Option<Vec<Entry>> {
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
                UnknownValue::Fixed32(v) => out.push(Entry {
                    path: nested_path,
                    value: EntryValue::Int((*v).into()),
                }),
                UnknownValue::Fixed64(v) => out.push(Entry {
                    path: nested_path,
                    value: EntryValue::Int((*v).into()),
                }),
                UnknownValue::Varint(v) => out.push(Entry {
                    path: nested_path,
                    value: EntryValue::Int(*v),
                }),
                UnknownValue::VariableLength(v) => {
                    if let Some(nested_entries) = try_parse_entries(&v, &nested_path) {
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
        let entries = try_parse_entries(b"\x12\x07Unknown", &[]).unwrap();
        assert_eq!(
            entries,
            &[Entry {
                path: vec![2],
                value: EntryValue::Bytes(b"Unknown".to_vec())
            }]
        );

        // two
        let entries = try_parse_entries(b"\x12\x07Unknown\x12\x07Unknown", &[]).unwrap();
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
        let entries = try_parse_entries(b"\x12\x07Unknown", &[42]).unwrap();
        assert_eq!(
            entries,
            &[Entry {
                path: vec![42, 2],
                value: EntryValue::Bytes(b"Unknown".to_vec())
            }]
        );

        // No valid protobuf (incomplete)
        let res = try_parse_entries(b"\x12\x07Unknown\x0a\x0fAtlantic ", &[]);
        assert_eq!(res, None);

        // No valid protobuf (wrong wire type)
        // End group (deprecated) in field 2: hex((2 << 3) | 4)
        let res = try_parse_entries(b"\x14\x07Unknown", &[]);
        assert_eq!(res, None);
    }

    #[test]
    fn try_parse_entries_returns_none_for_empty() {
        let res = try_parse_entries(b"", &[]);
        assert_eq!(res, None);
    }
}
