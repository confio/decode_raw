use ansi_term::Colour::{Green, Red};
use protofish::decode::UnknownValue;
use protofish::prelude::*;
use std::io::Read;

mod proto;

use proto::decode_fields;

fn main() {
    let mut input = Vec::<u8>::new();
    for byte in std::io::stdin().bytes() {
        input.push(byte.unwrap());
    }

    decode(&input);
}

fn decode(bytes: &[u8]) {
    if let Some(msg) = try_decode_message(bytes, 0) {
        print!("{}\n", msg);
    } else {
        panic!("Input bytes is not a valid protobuf serialization");
    }
}

fn try_decode_message(bytes: &[u8], indent: usize) -> Option<String> {
    if bytes.len() == 0 {
        return None;
    }

    let fields = decode_fields(&bytes);
    let mut out = String::new();
    for field in fields.into_iter() {
        if out.len() != 0 {
            out.push('\n');
        }
        for _ in 0..indent {
            out.push('·');
        }
        out.push_str(&format!("{}: ", field.number));
        match &field.value {
            Value::Unknown(unknown) => match unknown {
                UnknownValue::Fixed32(v) => out.push_str(&print_int(*v)),
                UnknownValue::Fixed64(v) => out.push_str(&print_int(*v)),
                UnknownValue::Varint(v) => out.push_str(&print_int(*v)),
                UnknownValue::VariableLength(v) => {
                    if let Some(nested_msg) = try_decode_message(&v, indent + 1) {
                        out.push('\n');
                        // out.push_str(&format!("(length {})", v.len()));
                        out.push_str(&nested_msg);
                    } else {
                        out.push_str(&format!("({} bytes) ", v.len()));
                        out.push_str(&print_bytes(v));
                    }
                }
                UnknownValue::Invalid(_wire_type, _bytes) => {
                    return None;
                }
            },
            _ => return None,
        }
    }
    Some(out)
}

fn print_int(i: impl Into<u128>) -> String {
    Red.paint(i.into().to_string()).to_string()
}

fn print_bytes(bytes: &[u8]) -> String {
    let text = match std::str::from_utf8(bytes) {
        Ok(converted) => format!("\"{}\"", converted),
        Err(_err) => hex::encode(bytes),
    };
    Green.paint(text).to_string()
}
