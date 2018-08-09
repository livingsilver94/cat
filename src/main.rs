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
		NonEmpty
	} else if opts.opt_present("n") {
		All
	} else {
		None
	};
	let end_char = if opts.opts_present(&['A'.to_string(), 'E'.to_string(), 'e'.to_string()]) {
		Some(String::from("$"))
	} else {
		Option::None
	};
	let tab_char = if opts.opts_present(&['A'.to_string(), 'T'.to_string(), 't'.to_string()]) {
		Some(String::from("^I"))
	} else {
		Option::None
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
	fn should_manipulate(&self) -> bool {
		self.numbering_mode != None
			|| self.end_char.is_some()
			|| self.squeeze_blank
			|| self.tab_char.is_some()
			|| self.show_nonprinting
	}
}

#[derive(PartialEq)]
enum NumberingMode {
	All,
	NonEmpty,
	None,
}

fn print_files(options: &CatOptions, filenames: &[&str]) -> Result<(), io::Error> {
	if !options.should_manipulate() {
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
					stdout_handle.write_all(
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
				// Check if we have to manipulate the line byte-by-byte
				if options.end_char.is_some() || options.show_nonprinting {
					for &byte in &buf {
						if let Some(ref tab_str) = options.tab_char {
							stdout_handle.write_all(tab_str.as_bytes())?;
						} else if options.show_nonprinting {
							stdout_handle.write_all(make_byte_printable(byte).as_bytes())?;
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

fn make_byte_printable(byte: u8) -> String {
	match byte {
		0...31 => format!("^{}", byte + 64),
		127 => "^?".to_string(),
		128...159 => format!("M-^{}", byte - 128),
		160...254 => format!("M-{}", byte - 128),
		255 => "M-^?".to_string(),
		32...126 | _ => unsafe { String::from_utf8_unchecked([byte].to_vec()) },
	}
}
