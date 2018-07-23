extern crate clap;

use clap::{App, Arg, ArgMatches};
use NumberingMode::*;

fn main() {
    let usr_input = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(Arg::with_name("show-all").short("A").long("show-all"))
        .arg(
            Arg::with_name("number-nonblank")
                .short("b")
                .long("number-nonblank"),
        )
        .arg(Arg::with_name("vE").short("e"))
        .arg(Arg::with_name("show-ends").short("E").long("show-ends"))
        .arg(Arg::with_name("number").short("n").long("number"))
        .arg(
            Arg::with_name("squeeze-blank")
                .short("s")
                .long("squeeze-blank"),
        )
        .arg(Arg::with_name("vT").short("t"))
        .arg(Arg::with_name("show-tabs").short("T").long("show-tabs"))
        .arg(
            Arg::with_name("show-nonprinting")
                .short("v")
                .long("show-nonprinting"),
        )
        .arg(Arg::with_name("files").multiple(true))
        .get_matches();
    let numbering_mode = if usr_input.is_present("number-nonblank") {
        NumberNonEmpty
    } else if usr_input.is_present("number") {
        NumberAll
    } else {
        NumberNone
    };
    let end_char = if has_any(&usr_input, &vec!["show-all", "show-ends", "vE"]) {
        Some(String::from("$"))
    } else {
        None
    };
    let tab_char = if has_any(&usr_input, &vec!["show-all", "show-tabs", "vT"]) {
        Some(String::from("^I"))
    } else {
        None
    };
    let options = CatOptions {
        numbering_mode,
        end_char,
        squeeze_blank: usr_input.is_present("squeeze-blank"),
        tab_char,
        show_nonprinting: has_any(&usr_input, &vec!["show-all", "show-nonprinting", "vE", "vT",]),
    };
    println!("{:?}", options);
}

#[derive(Debug)]
struct CatOptions {
    numbering_mode: NumberingMode,
    end_char: Option<String>,
    squeeze_blank: bool,
    tab_char: Option<String>,
    show_nonprinting: bool,
}

#[derive(Debug)]
enum NumberingMode {
    NumberAll,
    NumberNonEmpty,
    NumberNone,
}

fn has_any(args: &ArgMatches, opt_names: &[&str]) -> bool {
    for opt in opt_names {
        if args.is_present(opt) {
            return true;
        }
    }
    false
}
