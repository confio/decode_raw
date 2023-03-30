use clap::{ArgEnum, Parser};
use std::io::Read;

mod display;
mod filter;
mod parse;

use display::{dotted, escape_string, show_as, spaced, ColorMode, ShowAs};
use filter::{is_selected, SelectQuery};
use parse::{try_parse_entries, EntryValue, ParseConfig};

/// Simple program to greet a person
#[derive(Parser)]
#[clap(author, about, version, long_about = None)]
struct Args {
    /// Use colors in the output
    #[clap(arg_enum, long, default_value = "auto")]
    color: Color,

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
enum Color {
    Yes,
    Never,
    Auto,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum IndentStyle {
    Space,
    Dot,
    Path,
}

#[derive(Clone)]
struct Config {
    pub color: ColorMode,
    pub indent: IndentStyle,
    pub select: SelectQuery,
    pub full: bool,
    pub parse_config: ParseConfig,
}

fn is_a_tty() -> bool {
    unsafe { libc::isatty(libc::STDOUT_FILENO) }.is_positive()
}

fn main() {
    let args = Args::parse();

    let mut input = Vec::<u8>::new();
    for byte in std::io::stdin().bytes() {
        input.push(byte.unwrap());
    }

    let config = Config {
        color: match args.color {
            Color::Yes => ColorMode::Colored,
            Color::Never => ColorMode::Plain,
            Color::Auto => {
                if is_a_tty() {
                    ColorMode::Colored
                } else {
                    ColorMode::Plain
                }
            }
        },
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
    if let Some(entries) = try_parse_entries(bytes, config.parse_config) {
        for entry in entries
            .into_iter()
            .filter(|e| is_selected(e, &config.select))
        {
            let stripped_path = entry.path[config.select.len()..].to_vec();

            let path = print_path(&stripped_path, config);
            match entry.value {
                EntryValue::Fixed64(v) => {
                    println!("{}: (64 bit) {}", path, print_fixed64(v, config.color))
                }
                EntryValue::Fixed32(v) => {
                    println!("{}: (32 bit) {}", path, print_fixed32(v, config.color))
                }
                EntryValue::Varint(i) => println!("{}: {}", path, print_int(i, config.color)),
                EntryValue::Bytes(v) => {
                    print!(
                        "{}: ({} bytes) {}\n",
                        path,
                        v.len(),
                        print_bytes(&v, config.color, config.full)
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

fn print_fixed64(v: [u8; 8], color: ColorMode) -> String {
    let as_unsigned = u64::from_le_bytes(v);
    let as_signed = i64::from_le_bytes(v);
    let as_float = f64::from_le_bytes(v);

    let mut values = Vec::<String>::new();
    values.push(color.yellow(as_unsigned.to_string()));
    if as_signed < 0 {
        values.push(color.yellow(as_signed.to_string()));
    }
    values.push(color.yellow(as_float.to_string()));
    values.join(" / ")
}

fn print_fixed32(v: [u8; 4], color: ColorMode) -> String {
    let as_unsigned = u32::from_le_bytes(v);
    let as_signed = i32::from_le_bytes(v);
    let as_float = f32::from_le_bytes(v);

    let mut values = Vec::<String>::new();
    values.push(color.yellow(as_unsigned.to_string()));
    if as_signed < 0 {
        values.push(color.yellow(as_signed.to_string()));
    }
    values.push(color.yellow(as_float.to_string()));
    values.join(" / ")
}

fn print_int(i: impl Into<u128>, color: ColorMode) -> String {
    color.red(i.into().to_string())
}

fn print_bytes(bytes: &[u8], color: ColorMode, full: bool) -> String {
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
    color.green(text)
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
