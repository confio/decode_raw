use ansi_term::Colour::{Green, Red};
use std::io::Read;

mod indent;
mod proto;
mod select;

use indent::{dotted, spaced};
use proto::try_parse_entries;
use select::parse_select;

use clap::{ArgEnum, Parser};

use crate::proto::EntryValue;

/// Simple program to greet a person
#[derive(Parser)]
#[clap(author, about, long_about = None)]
struct Args {
    /// How to style indent
    #[clap(arg_enum, short, long, default_value = "dot")]
    indent: IndentStyle,

    /// The path to select. e.g. .2.1.1
    #[clap()]
    select: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum IndentStyle {
    Space,
    Dot,
    Path,
}

#[derive(Clone)]
struct Config {
    pub indent: IndentStyle,
    pub select: Vec<u64>,
}

fn main() {
    let args = Args::parse();

    let mut input = Vec::<u8>::new();
    for byte in std::io::stdin().bytes() {
        input.push(byte.unwrap());
    }

    let config = Config {
        indent: args.indent,
        select: parse_select(&args.select.unwrap_or_default()),
    };

    decode(&input, &config);
}

fn decode(bytes: &[u8], config: &Config) {
    if let Some(entries) = try_parse_entries(bytes, &[]) {
        for entry in entries {
            if !entry.path.starts_with(&config.select) {
                continue;
            }

            let stripped_path = entry.path[config.select.len()..].to_vec();

            let path = print_path(&stripped_path, config);
            match entry.value {
                EntryValue::Int(i) => print!("{}: {}\n", path, print_int(i)),
                EntryValue::Bytes(v) => {
                    print!("{}: ({} bytes) {}\n", path, v.len(), print_bytes(&v))
                }
                EntryValue::OpenNested => {
                    if !path.is_empty() {
                        print!("{} {{\n", path);
                    }
                }
                EntryValue::CloseNested => {
                    if !path.is_empty() {
                        print!("{}}}\n", dotted((path.chars().count() - 1) / 2));
                    }
                }
            }
        }
    } else {
        panic!("Input bytes is not a valid protobuf serialization");
    }
}

fn print_int(i: impl Into<u128>) -> String {
    Red.paint(i.into().to_string()).to_string()
}

fn print_bytes(bytes: &[u8]) -> String {
    let mut text = match std::str::from_utf8(bytes) {
        Ok(converted) => format!("\"{}\"", converted),
        Err(_err) => hex::encode(bytes),
    };
    const MAX_CHARS: usize = 500;
    if text.chars().take(MAX_CHARS + 1).count() > MAX_CHARS {
        let mut truncated: String = text.chars().take(MAX_CHARS).collect();
        truncated.push('…');
        text = truncated;
    }
    Green.paint(text).to_string()
}

fn print_path(path: &[u64], config: &Config) -> String {
    match config.indent {
        IndentStyle::Dot => {
            let mut out = dotted(path.len().saturating_sub(1));
            if let Some(last) = path.last() {
                out.push_str(&format!("{}", last));
            }
            out
        }
        IndentStyle::Space => {
            let mut out = spaced(path.len().saturating_sub(1));
            if let Some(last) = path.last() {
                out.push_str(&format!("{}", last));
            }
            out
        }
        IndentStyle::Path => {
            let formated_path: String = path.iter().map(|number| format!(".{}", number)).collect();
            formated_path
        }
    }
}
