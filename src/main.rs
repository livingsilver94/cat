extern crate getopts;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use NumberingMode::*;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let opts = getopts::Options::new()
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
        .parse(&args)
        .unwrap();

    let numbering_mode = if opts.opt_present("b") {
        NumberNonEmpty
    } else if opts.opt_present("n") {
        NumberAll
    } else {
        NumberNone
    };
    let end_char = if opts.opts_present(&['A'.to_string(), 'E'.to_string(), 'e'.to_string()]) {
        Some(String::from("$"))
    } else {
        None
    };
    let tab_char = if opts.opts_present(&['A'.to_string(), 'T'.to_string(), 't'.to_string()]) {
        Some(String::from("^I"))
    } else {
        None
    };
    let options = CatOptions {
        numbering_mode,
        end_char,
        squeeze_blank: opts.opt_present("s"),
        tab_char,
        show_nonprinting: opts.opts_present(&[
            'A'.to_string(),
            'v'.to_string(),
            'e'.to_string(),
            't'.to_string(),
        ]),
    };
    let files: Vec<&str> = opts.free.iter().map(|x| &x[..]).collect();
    print_files(&options, &files);
}

struct CatOptions {
    numbering_mode: NumberingMode,
    end_char: Option<String>,
    squeeze_blank: bool,
    tab_char: Option<String>,
    show_nonprinting: bool,
}

impl CatOptions {
    fn must_read_by_line(&self) -> bool {
        self.numbering_mode != NumberNone
            || self.end_char.is_some()
            || self.squeeze_blank
            || self.tab_char.is_some()
            || self.show_nonprinting
    }
}

#[derive(PartialEq)]
enum NumberingMode {
    NumberAll,
    NumberNonEmpty,
    NumberNone,
}

fn print_files(options: &CatOptions, filenames: &[&str]) -> Result<(), io::Error> {
    if !options.must_read_by_line() {
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
                if options.numbering_mode == NumberAll
                    || (options.numbering_mode == NumberNonEmpty && !is_blank(&buf))
                {
                    stdout_handle.write(
                        format!(
                            "{}{}\t",
                            &"     "[(number_of_digits(line) as usize) - 1..],
                            line
                        ).as_bytes(),
                    )?;
                    line += 1;
                }
                if let Some(ref chr) = options.end_char {
                    if buf[len - 1] == b'\n' {
                        append_str(&mut buf, &chr);
                    }
                }
                if let Some(ref repl) = options.tab_char {
                    for &byte in &buf {
                        if byte == b'\t' {
                            stdout_handle.write(repl.as_bytes())?;
                        } else {
                            stdout_handle.write(&[byte])?;
                        }
                    }
                } else {
                    stdout_handle.write(&buf)?;
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
    line.len() <= 2 && line[0] == b'\n'
}

/// Insert a string before newline character
fn append_str(line: &mut Vec<u8>, item: &str) {
    let mut index = line.len() - 1;
    for chr in item.chars() {
        line.insert(index, chr as u8);
        index += 1;
    }
}

fn number_of_digits(int: u64) -> u32 {
    match int {
        0...9 => 1,
        10...99 => 2,
        100...999 => 3,
        1000...9999 => 4,
        10000...99999 => 5,
        _ => 6,
    }
}
