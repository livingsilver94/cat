extern crate cat;
extern crate getopts;

use cat::NumberingMode::{All, NonEmpty};
use cat::{CatOptions, NumberingMode, Optionify};
use std::env;
use std::process;

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
        println!(
            "{}",
            opts.usage("Usage: cat [OPTION]... [FILE]...\nConcatenate FILE(s) to standard output.")
        );
    } else {
        let numbering_mode = if matches.opt_present("b") {
            NonEmpty
        } else if matches.opt_present("n") {
            All
        } else {
            NumberingMode::None
        };
        let end_char = matches
            .opts_present(&['A'.to_string(), 'E'.to_string(), 'e'.to_string()])
            .as_some("$".to_string());
        let tab_char = matches
            .opts_present(&['A'.to_string(), 'E'.to_string(), 'e'.to_string()])
            .as_some("^I".to_string());
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
        if let Err(error) = cat::print_files(&options, &files) {
            match error.get_ref() {
                Some(inner) => {
                    eprintln!("{}", inner);
                    process::exit(1);
                }
                None => {
                    eprintln!("{}", error);
                    process::exit(error.raw_os_error().unwrap());
                }
            }
        }
    }
}
