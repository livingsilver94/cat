extern crate getopts;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use NumberingMode::*;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut opts = getopts::Options::new();
    let opts = opts
        .optflag("A", "show-all", "equivalent to -vET")
        .optflag(
            "b",
            "number-nonblank",
            "number nonempty output lines, overrides -n",
        )
        .optflag("e", "", "equivalent to -vE")
        .optflag("E", "show-ends", "display $ at end of each line")
        .optflag("n", "number", "number all output lines")
        .optflag("s", "squeeze-blank", "suppress repeated empty output lines")
        .optflag("t", "", "equivalent to -vT")
        .optflag("T", "show-tabs", "display TAB characters as ^I")
        .optflag(
            "v",
            "show-nonprinting",
            "use ^ and M- notation, except for LFD and TAB",
        )
        .optflag("h", "help", "display this help and exit");
    let matches = opts.parse(&args).unwrap();
    if matches.opt_present("h") {
        println!("{}", opts.usage("Usage: cat [OPTION]... [FILE]...\nConcatenate FILE(s) to standard output."));
    } else {
        let numbering_mode = if matches.opt_present("b") {
            NonEmpty
        } else if matches.opt_present("n") {
            All
        } else {
            None
        };
        let end_char = if matches.opts_present(&['A'.to_string(), 'E'.to_string(), 'e'.to_string()])
        {
            Some(String::from("$"))
        } else {
            Option::None
        };
        let tab_char = if matches.opts_present(&['A'.to_string(), 'T'.to_string(), 't'.to_string()])
        {
            Some(String::from("^I"))
        } else {
            Option::None
        };
        let options = CatOptions {
            numbering_mode,
            end_char,
            squeeze_blank: matches.opt_present("s"),
            tab_char,
            show_nonprinting: matches.opts_present(&[
                'A'.to_string(),
                'v'.to_string(),
                'e'.to_string(),
                't'.to_string(),
            ]),
        };
        let files: Vec<&str> = matches.free.iter().map(|x| &x[..]).collect();
        print_files(&options, &files);
    }
}

struct CatOptions {
    numbering_mode: NumberingMode,
    end_char: Option<String>,
    squeeze_blank: bool,
    tab_char: Option<String>,
    show_nonprinting: bool,
}

#[derive(PartialEq)]
enum NumberingMode {
    All,
    NonEmpty,
    None,
}

fn print_files(options: &CatOptions, filenames: &[&str]) -> Result<(), io::Error> {
    // Check if we can print files without any manipulation (hence faster)
    if options.numbering_mode == None
        && options.end_char.is_none()
        && !options.squeeze_blank
        && options.tab_char.is_none()
        && !options.show_nonprinting
    {
        fast_print(&filenames)?;
    } else {
        let stdout = io::stdout();
        let mut stdout_handle = stdout.lock();
        let mut buf = Vec::with_capacity(64);
        let mut line = 1;
        let mut was_blank = false;
        for path in filenames {
            let mut buf_reader = io::BufReader::new(open_file(path)?);
            while let Ok(ref len) = buf_reader.read_until(b'\n', &mut buf) {
                if *len == 0 {
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
                    // Once u64::to_bytes is out from nightly, we will be able to
                    // avoid that String allocation and print bytes directly into the buffer
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
                        if let Some(ref tab_str) = options.tab_char {
                            stdout_handle.write_all(tab_str.as_bytes())?;
                        } else if options.show_nonprinting {
                            match byte {
                                0...8 | 11...31 => stdout_handle.write_all(&[b'^', byte + 64])?,
                                127 => stdout_handle.write_all(&[b'^', b'?'])?,
                                128...159 => {
                                    stdout_handle.write_all(&[b'M', b'-', b'^', byte - 64])?
                                }
                                160...254 => stdout_handle.write_all(&[b'M', b'-', byte - 128])?,
                                255 => stdout_handle.write_all(&[b'M', b'-', b'^', b'?'])?,
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
fn fast_print(filenames: &[&str]) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut stdout_handle = stdout.lock();
    for path in filenames {
        let mut file = open_file(path)?;
        io::copy(&mut file, &mut stdout_handle)?;
    }
    Ok(())
}

fn open_file(path: &str) -> Result<Box<Read>, io::Error> {
    Ok(if path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(path)?)
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
