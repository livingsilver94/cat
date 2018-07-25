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
    let end_char = if opts.opts_present(&["A".to_string(), "E".to_string(), "e".to_string()]) {
        Some(String::from("$"))
    } else {
        None
    };
    let tab_char = if opts.opts_present(&["A".to_string(), "T".to_string(), "t".to_string()]) {
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

#[derive(Debug)]
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

#[derive(Debug, PartialEq)]
enum NumberingMode {
    NumberAll,
    NumberNonEmpty,
    NumberNone,
}

fn print_files(options: &CatOptions, filenames: &[&str]) {
    let mut buffer = [0; 1024 * 64];
    if !options.must_read_by_line() {
        for path in filenames {
            let mut file: Box<Read> = if *path == "-" {
                Box::new(io::stdin()) as Box<Read>
            } else {
                Box::new(File::open(path).unwrap())
            };
            file.read(&mut buffer);
            io::stdout().write(&buffer);
        }
    }
}
