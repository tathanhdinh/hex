#![deny(
    // missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
// #![allow(dead_code)]

//! general hex lib
extern crate ansi_term;
extern crate clap;
extern crate failure;

use clap::ArgMatches;
use failure::Fail;
use std::{
    f64,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
    result,
};

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "IO error: {}", _0)]
    Io(#[cause] io::Error),

    // #[fail(display = "Application error: {}", _0)]
    // Application(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

pub(crate) type Result<T> = result::Result<T, Error>;

/// nothing ⇒ Display
/// ? ⇒ Debug
/// o ⇒ Octal
/// x ⇒ LowerHex
/// X ⇒ UpperHex
/// p ⇒ Pointer
/// b ⇒ Binary
/// e ⇒ LowerExp
/// E ⇒ UpperExp
/// evaulate for traits implementation
/// https://stackoverflow.com/questions/27650312/show-u8-slice-in-hex-representation
#[derive(Copy, Clone, Debug)]
pub enum Format {
    /// octal format
    Octal,
    /// lower hex format
    LowerHex,
    /// upper hex format
    UpperHex,
    /// pointer format
    Pointer,
    /// binary format
    Binary,
    /// lower exp format
    LowerExp,
    /// upper exp format
    UpperExp,
    /// unknown format
    Unknown,
}

/// Line structure for hex output
#[derive(Clone, Debug)]
pub struct Line {
    /// offset
    pub offset: u64,
    /// hex body
    pub hex_body: Vec<u8>,
    /// ascii text
    pub ascii: Vec<char>,
    /// total bytes in Line
    pub bytes: u64,
}
/// Line implementation
impl Line {
    /// Line constructor
    pub fn new() -> Line {
        Line {
            offset: 0x0,
            hex_body: Vec::new(),
            ascii: Vec::new(),
            bytes: 0x0,
        }
    }
}

/// Page structure
#[derive(Clone, Debug)]
pub struct Page {
    /// page offset
    pub offset: u64,
    /// page body
    pub body: Vec<Line>,
    /// total bytes in page
    pub bytes: u64,
}

/// Page implementation
impl Page {
    /// Page constructor
    pub fn new() -> Page {
        Page {
            offset: 0x0,
            body: Vec::new(),
            bytes: 0x0,
        }
    }
}

/// offset column
///
/// # Arguments
///
/// * `b` - offset value.
pub fn offset(b: u64) -> String {
    format!("{:#08x}", b)
}

/// print offset to std out
pub fn print_offset(b: u64) {
    print!("{}: ", offset(b));
}

/// hex octal, takes u8
pub fn hex_octal(b: u8) -> String {
    format!("{:#06o}", b)
}

/// hex lower hex, takes u8
pub fn hex_lower_hex(b: u8) -> String {
    format!("{:#04x}", b)
}

/// hex upper hex, takes u8
pub fn hex_upper_hex(b: u8) -> String {
    format!("{:#04X}", b)
}

/// hex binary, takes u8
pub fn hex_binary(b: u8) -> String {
    format!("{:#010b}", b)
}

/// print byte to std out
pub fn print_byte<T: Write>(b: u8, format: Format, colorize: bool, w: &mut T) -> Result<()> {
    let mut color: u8 = b;
    if color < 1 {
        color = 0x16;
    }

    let write_result = if colorize {
        // note, for color testing: for (( i = 0; i < 256; i++ )); do echo "$(tput setaf $i)This is ($i) $(tput sgr0)"; done
        match format {
            Format::Octal => write!(
                w,
                "{} ",
                ansi_term::Style::new()
                    .fg(ansi_term::Color::Fixed(color))
                    .paint(hex_octal(b))
            ),
            Format::LowerHex => write!(
                w,
                "{} ",
                ansi_term::Style::new()
                    .fg(ansi_term::Color::Fixed(color))
                    .paint(hex_lower_hex(b))
            ),
            Format::UpperHex => write!(
                w,
                "{} ",
                ansi_term::Style::new()
                    .fg(ansi_term::Color::Fixed(color))
                    .paint(hex_upper_hex(b))
            ),
            Format::Binary => write!(
                w,
                "{} ",
                ansi_term::Style::new()
                    .fg(ansi_term::Color::Fixed(color))
                    .paint(hex_binary(b))
            ),
            _ => write!(w, "{}", "unk_fmt "),
        }
    } else {
        match format {
            Format::Octal => write!(w, "{} ", hex_octal(b)),
            Format::LowerHex => write!(w, "{} ", hex_lower_hex(b)),
            Format::UpperHex => write!(w, "{} ", hex_upper_hex(b)),
            Format::Binary => write!(w, "{} ", hex_binary(b)),
            _ => write!(w, "{}", "unk_fmt "),
        }
    };

    write_result.map_err(Error::Io)
}

/// Function wave out.
/// # Arguments
///
/// * `len` - Wave length.
/// * `places` - Number of decimal places for function wave floats.
pub fn func_out(len: u64, places: usize) {
    for y in 0..len {
        let y_float: f64 = y as f64;
        let len_float: f64 = len as f64;
        let x: f64 = (((y_float / len_float) * f64::consts::PI) / 2.0).sin();
        let formatted_number = format!("{:.*}", places, x);
        print!("{}", formatted_number);
        print!(",");
        if (y % 10) == 9 {
            println!("");
        }
    }
    println!("");
}

/// In most hex editor applications, the data of the computer file is
/// represented as hexadecimal values grouped in 4 groups of 4 bytes
/// (or two groups of 8 bytes), followed by one group of 16 printable ASCII
/// characters which correspond to each pair of hex values (each byte).
/// Non-printable ASCII characters (e.g., Bell) and characters that would take
/// more than one character space (e.g., tab) are typically represented by a
/// dot (".") in the following ASCII field.
///
/// # Arguments
///
/// * `matches` - Argument matches from command line.
pub fn run(matches: ArgMatches) -> Result<()> {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stdout = BufWriter::new(stdout);

    let mut column_width: u64 = 10;
    if let Some(len) = matches.value_of("func") {
        let mut p: usize = 4;
        if let Some(places) = matches.value_of("places") {
            p = places.parse::<usize>().unwrap();
        }
        func_out(len.parse::<u64>().unwrap(), p);
    } else if let Some(file) = matches.value_of("INPUTFILE") {
        let f = File::open(file).unwrap();
        let mut buf_len = fs::metadata(file)?.len();
        let mut buf = BufReader::new(f);
        let mut format_out = Format::LowerHex;
        let mut colorize = true;

        if let Some(columns) = matches.value_of("cols") {
            column_width = columns.parse::<u64>().unwrap(); //turbofish
        }

        if let Some(length) = matches.value_of("len") {
            buf_len = length.parse::<u64>().unwrap();
        }

        if let Some(format) = matches.value_of("format") {
            // o, x, X, p, b, e, E
            match format {
                "o" => format_out = Format::Octal,
                "x" => format_out = Format::LowerHex,
                "X" => format_out = Format::UpperHex,
                "p" => format_out = Format::Pointer,
                "b" => format_out = Format::Binary,
                "e" => format_out = Format::LowerExp,
                "E" => format_out = Format::UpperExp,
                _ => format_out = Format::Unknown,
            }
        }

        if let Some(color) = matches.value_of("color") {
            let color_v = color.parse::<u8>().unwrap();
            if color_v == 1 {
                colorize = true;
            } else {
                colorize = false;
            }
        }

        match matches.occurrences_of("v") {
            0 => write!(&mut stdout, "")?,
            1 => write!(&mut stdout, "verbose 1")?,
            2 => write!(&mut stdout, "verbose 2")?,
            3 | _ => write!(&mut stdout, "verbose max")?,
        }

        // array output mode is mutually exclusive
        if let Some(array) = matches.value_of("array") {
            let mut array_format = array;
            let mut page = buf_to_array(&mut buf, buf_len, column_width).unwrap();
            match array_format {
                "r" => writeln!(&mut stdout, "let ARRAY: [u8; {}] = [", page.bytes)?,
                "c" => writeln!(&mut stdout, "unsigned char ARRAY[{}] = {{", page.bytes)?,
                "g" => writeln!(&mut stdout, "a := [{}]byte{{", page.bytes)?,
                _ => writeln!(&mut stdout, "unknown array format")?,
            }

            let mut i: u64 = 0x0;
            for line in page.body.iter() {
                write!(&mut stdout, "    ");
                for hex in line.hex_body.iter() {
                    i += 1;
                    if i == buf_len && array_format != "g" {
                        write!(&mut stdout, "{}", hex_lower_hex(*hex));
                    } else {
                        write!(&mut stdout,"{}, ", hex_lower_hex(*hex));
                    }
                }
                writeln!(&mut stdout, "");
            }
            match array_format {
                "r" => writeln!(&mut stdout, "{}", "];")?,
                "c" => writeln!(&mut stdout, "{}", "};")?,
                "g" => writeln!(&mut stdout, "{}", "}")?,
                _ => writeln!(&mut stdout, "unknown array format")?,
            }
        } else {
            // Transforms this Read instance to an Iterator over its bytes.
            // The returned type implements Iterator where the Item is
            // Result<u8, R::Err>. The yielded item is Ok if a byte was
            // successfully read and Err otherwise for I/O errors. EOF is mapped
            // to returning None from this iterator.
            // (https://doc.rust-lang.org/1.16.0/std/io/trait.Read.html#method.bytes)
            let mut ascii_line: Line = Line::new();
            let mut offset_counter: u64 = 0x0;
            let mut byte_column: u64 = 0x0;
            let mut page = buf_to_array(&mut buf, buf_len, column_width).unwrap();

            for line in page.body.iter() {
                print_offset(offset_counter);

                for hex in line.hex_body.iter() {
                    offset_counter += 1;
                    byte_column += 1;
                    print_byte(*hex, format_out, colorize, &mut stdout)?;

                    if *hex > 31 && *hex < 127 {
                        ascii_line.ascii.push(*hex as char);
                    } else {
                        ascii_line.ascii.push('.');
                    }
                }

                if byte_column < column_width {
                    write!(&mut stdout, "{:<1$}", "", 5 * (column_width - byte_column) as usize);
                }

                byte_column = 0x0;
                let ascii_string: String = ascii_line.ascii.iter().cloned().collect();
                ascii_line = Line::new();
                write!(&mut stdout, "{}", ascii_string); // print ascii string
                writeln!(&mut stdout, "");
            }
            if true {
                writeln!(&mut stdout, "   bytes: {}", page.bytes);
            }
        }
    }
    Ok(())
}

/// Buffer to array.
///
/// (https://rustbyexample.com/primitives/array.html)
/// (https://stackoverflow.com/questions/39464237/whats-the-idiomatic-way-reference-bufreader-bufwriter-when-passing-between-funct)
/// (https://stackoverflow.com/questions/39935158/bufreader-move-after-for-loop-with-bufreader-lines)
///
/// # Arguments
///
/// * `buf` - Buffer to be read.
/// * `buf_len` - Buffer length.
/// * `column_width` - column width for output.
pub fn buf_to_array(
    buf: &mut Read,
    buf_len: u64,
    column_width: u64,
) -> Result<Page> {
    let mut column_count: u64 = 0x0;
    let max_array_size: u16 = <u16>::max_value(); // 2^16;
    let mut page: Page = Page::new();
    let mut line: Line = Line::new();
    for b in buf.bytes() {
        let b1: u8 = b.unwrap();
        line.bytes += 1;
        page.bytes += 1;
        line.hex_body.push(b1);
        column_count += 1;

        if column_count >= column_width {
            page.body.push(line);
            line = Line::new();
            column_count = 0;
        }
        if page.bytes == buf_len || max_array_size as u64 == buf_len {
            page.body.push(line);
            break;
        }
    }
    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// @see (https://users.rust-lang.org/t/how-to-test-output-to-stdout/4877/6)
    /// @see (https://rustbyexample.com/hello/print/print_display.html)
    #[test]
    fn test_offset() {
        let b: u64 = 0x6;
        assert_eq!(offset(b), "0x000006");
        assert_eq!(offset(b), format!("{:#08x}", b));
    }

    /// hex octal, takes u8
    #[test]
    pub fn test_hex_octal() {
        let b: u8 = 0x6;
        assert_eq!(hex_octal(b), "0o0006");
        assert_eq!(hex_octal(b), format!("{:#06o}", b));
    }

    /// hex lower hex, takes u8
    #[test]
    fn test_hex_lower_hex() {
        let b: u8 = <u8>::max_value(); // 255
        assert_eq!(hex_lower_hex(b), "0xff");
        assert_eq!(hex_lower_hex(b), format!("{:#04x}", b));
    }

    /// hex upper hex, takes u8
    #[test]
    fn test_hex_upper_hex() {
        let b: u8 = <u8>::max_value();
        assert_eq!(hex_upper_hex(b), "0xFF");
        assert_eq!(hex_upper_hex(b), format!("{:#04X}", b));
    }

    /// hex binary, takes u8
    #[test]
    fn test_hex_binary() {
        let b: u8 = <u8>::max_value();
        assert_eq!(hex_binary(b), "0b11111111");
        assert_eq!(hex_binary(b), format!("{:#010b}", b));
    }
}
