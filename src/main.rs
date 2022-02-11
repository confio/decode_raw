use ansi_term::Colour::{Green, Red};
use clap::{ArgEnum, Parser};
use std::io::Read;

mod display;
mod filter;
mod parse;

use display::{dotted, escape_string, show_as, spaced, ShowAs};
use filter::SelectQuery;
use parse::{try_parse_entries, EntryValue, ParseConfig};

/// Simple program to greet a person
#[derive(Parser)]
#[clap(author, about, version, long_about = None)]
struct Args {
    /// How to style indent
    #[clap(arg_enum, short, long, default_value = "dot")]
    indent: IndentStyle,

    /// Assume wire type 1 or 5 (fixed64, sfixed64, double, fixed32, sfixed32, float) is not used.
    /// Implies --no_fixed64 and --no_fixed32.
    /// This helps auto-detecting bytes vs. message field.
    #[clap(long)]
    no_fixed: bool,

    /// Assume wire type 1 (fixed64, sfixed64, double) is not used.
    /// This helps auto-detecting bytes vs. message field.
    #[clap(long)]
    no_fixed64: bool,

    /// Assume wire type 5 (fixed32, sfixed32, float) is not used.
    /// This helps auto-detecting bytes vs. message field.
    #[clap(long)]
    no_fixed32: bool,

    /// Show all data in full length
    #[clap(long)]
    full: bool,

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
    pub select: SelectQuery,
    pub full: bool,
    pub parse_config: ParseConfig,
}

fn main() {
    let args = Args::parse();

    let mut input = Vec::<u8>::new();
    for byte in std::io::stdin().bytes() {
        input.push(byte.unwrap());
    }

    let config = Config {
        indent: args.indent,
        select: SelectQuery::parse(&args.select.unwrap_or_default()).unwrap(),
        full: args.full,
        parse_config: ParseConfig {
            no_fixed64: args.no_fixed || args.no_fixed64,
            no_fixed32: args.no_fixed || args.no_fixed32,
        },
    };

    decode(&input, &config);
}

fn decode(bytes: &[u8], config: &Config) {
    if let Some(entries) = try_parse_entries(bytes, &[], config.parse_config) {
        for entry in entries {
            if !entry.path.starts_with(&config.select) {
                continue;
            }

            let stripped_path = entry.path[config.select.len()..].to_vec();

            let path = print_path(&stripped_path, config);
            match entry.value {
                EntryValue::Int(i) => print!("{}: {}\n", path, print_int(i)),
                EntryValue::Bytes(v) => {
                    print!(
                        "{}: ({} bytes) {}\n",
                        path,
                        v.len(),
                        print_bytes(&v, config.full)
                    )
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

fn print_bytes(bytes: &[u8], full: bool) -> String {
    let text = match show_as(bytes) {
        ShowAs::String(s) => escape_string(s),
        ShowAs::Bytes(bytes) => {
            const MAX_BYTES: usize = 256;
            if full || bytes.len() <= MAX_BYTES {
                hex::encode(bytes)
            } else {
                let mut truncated = hex::encode(&bytes[0..MAX_BYTES]);
                truncated.push('…');
                truncated
            }
        }
    };
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
