extern crate clap;

use clap::{App, Arg};

fn main() {
    let usr_input = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(Arg::with_name("show-all").short("A").long("show-all"))
        .arg(Arg::with_name("number-nonblank").short("b").long("number-nonblank"))
        .arg(Arg::with_name("vE").short("e"))
        .arg(Arg::with_name("show-ends").short("E").long("show-ends"))
        .arg(Arg::with_name("number").short("n").long("number"))
        .arg(Arg::with_name("squeeze-blank").short("s").long("squeeze-blank"))
        .arg(Arg::with_name("vT").short("t"))
        .arg(Arg::with_name("show-tabs").short("T").long("show-tabs"))
        .arg(Arg::with_name("show-nonprinting").short("v").long("show-nonprinting"))
        .arg(Arg::with_name("files").multiple(true))
        .get_matches();
}

struct CatOptions {
    numbering_mode: NumberingMode,
    end_char: Option<String>,
    squeeze_blank: bool,
    tab_char: Option<String>,
    show_nonprinting: bool
}

enum NumberingMode {
    NumberAll,
    NumberNonEmpty,
    NumberNone
}