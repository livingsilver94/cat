extern crate failure;
#[macro_use]
extern crate failure_derive;

use failure::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use NumberingMode::{All, NonEmpty};

pub struct CatOptions {
    pub numbering_mode: NumberingMode,
    pub end_char: Option<String>,
    pub squeeze_blank: bool,
    pub tab_char: Option<String>,
    pub show_nonprinting: bool,
}

#[derive(PartialEq)]
pub enum NumberingMode {
    All,
    NonEmpty,
    None,
}

pub fn concat(options: &CatOptions, filenames: &[&str]) -> Result<(), Error> {
    // Check if we can print files without any manipulation (hence faster)
    if options.numbering_mode == NumberingMode::None
        && options.end_char.is_none()
        && !options.squeeze_blank
        && options.tab_char.is_none()
        && !options.show_nonprinting
    {
        fast_print(&filenames)?;
    } else {
        let stdout = io::stdout();
        let stdout_lock = stdout.lock();
        let mut stdout_handle = BufWriter::new(stdout_lock);
        let mut buf = Vec::with_capacity(64);
        let mut line = 1;
        let mut was_blank = false;
        for path in filenames {
            let mut buf_reader = io::BufReader::new(open_file(path)?);
            while let Ok(len) = buf_reader.read_until(b'\n', &mut buf) {
                if len == 0 {
                    break;
                }
                if options.squeeze_blank {
                    if was_blank && is_blank(&buf) {
                        buf.clear();
                        continue;
                    }
                    was_blank = is_blank(&buf);
                }
                if options.numbering_mode == All
                    || (options.numbering_mode == NonEmpty && !is_blank(&buf))
                {
                    stdout_handle
                        .write_all(format!("{}{}\t", numbering_prefix(line), line).as_bytes())?;
                    line += 1;
                }
                if let Some(ref chr) = options.end_char {
                    if buf[len - 1] == b'\n' {
                        append_str(&mut buf, &chr);
                    }
                }
                // Check if we have to manipulate the line byte-by-byte
                if options.end_char.is_some() || options.show_nonprinting {
                    for &byte in &buf {
                        if let (b'\t', Some(tab_str)) = (byte, &options.tab_char) {
                            stdout_handle.write_all(tab_str.as_bytes())?;
                        } else if options.show_nonprinting {
                            match byte {
                                0...8 | 11...31 => stdout_handle.write_all(&[b'^', byte + 64])?,
                                127 => stdout_handle.write_all(b"^?")?,
                                128...159 => {
                                    stdout_handle.write_all(&[b'M', b'-', b'^', byte - 64])?
                                }
                                160...254 => stdout_handle.write_all(&[b'M', b'-', byte - 128])?,
                                255 => stdout_handle.write_all(b"M-^?")?,
                                _ => stdout_handle.write_all(&[byte])?,
                            };
                        } else {
                            stdout_handle.write_all(&[byte])?;
                        }
                    }
                } else {
                    // No, we don't
                    stdout_handle.write_all(&buf)?;
                }
                buf.clear();
            }
        }
    }
    Ok(())
}

/// Print a list of file as-is, without any manipulation
pub fn fast_print(filenames: &[&str]) -> Result<(), Error> {
    let stdout = io::stdout();
    let mut stdout_handle = stdout.lock();
    for path in filenames {
        let mut file = open_file(path)?;
        io::copy(&mut file, &mut stdout_handle).with_filename(path.to_string())?;
    }
    Ok(())
}

fn open_file(path: &str) -> Result<Box<Read>, Error> {
    Ok(if path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(path).with_filename(path.to_string())?)
    })
}

fn is_blank(line: &[u8]) -> bool {
    line[0] == b'\n'
}

/// Insert a string before newline character
fn append_str(line: &mut Vec<u8>, item: &str) {
    let mut index = line.len() - 1;
    for chr in item.bytes() {
        line.insert(index, chr);
        index += 1;
    }
}

fn numbering_prefix(line_number: u64) -> &'static str {
    let spaces = match line_number {
        0...9 => 1,
        10...99 => 2,
        100...999 => 3,
        1000...9999 => 4,
        10000...99999 => 5,
        _ => 6,
    };
    &"     "[spaces - 1..]
}

#[derive(Fail, Debug)]
struct FileError {
    filename: String,
    #[cause]
    io_error: io::Error,
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.filename, self.io_error)
    }
}

trait ExtError<O> {
    fn with_filename(self, filename: String) -> Result<O, FileError>;
}

impl<O> ExtError<O> for io::Result<O> {
    fn with_filename(self, filename: String) -> Result<O, FileError> {
        match self {
            Ok(val) => Ok(val),
            Err(io_error) => Err(FileError { filename, io_error }),
        }
    }
}
