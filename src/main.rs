extern crate clap;
extern crate fish;

use clap::{App, Arg};
use std::process;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new(NAME)
        .version(VERSION)
        .author("Marc Noirot <marc.noirot@gmail.com>")
        .about("Fish language interpreter")
        .arg(
            Arg::with_name("INPUT")
                .help("set the input file to use")
                .conflicts_with("code")
                .value_name("FILE")
                .index(1),
        )
        .arg(
            Arg::with_name("code")
                .short("c")
                .long("code")
                .help("string of instructions to execute instead of FILE")
                .takes_value(true)
                .value_name("CODE"),
        )
        .arg(
            Arg::with_name("string")
                .short("s")
                .long("string")
                .help("push strings onto the stack before execution starts")
                .takes_value(true)
                .value_name("STRING")
                .multiple(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("value")
                .short("v")
                .long("value")
                .help("push numbers onto the stack before execution starts")
                .takes_value(true)
                .value_name("NUMBER")
                .multiple(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("tick")
                .short("t")
                .long("tick")
                .help("define a delay between the execution of each instruction")
                .takes_value(true)
                .value_name("DELAY"),
        )
        .arg(
            Arg::with_name("always_tick")
                .short("a")
                .long("always-tick")
                .help(
                    "make every instruction cause a tick, even whitespace and skipped instructions",
                ),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("dump interpreter state before executing an instruction"),
        )
        .get_matches();

    let code_box = match matches.value_of("code") {
        Some(c) => fish::CodeBox::load_from_string(&c),
        None => {
            let input = match matches.value_of("INPUT") {
                Some(v) => v,
                None => {
                    println!("{}", matches.usage());
                    process::exit(1)
                }
            };

            match fish::CodeBox::load_from_file(&input) {
                Ok(cb) => cb,
                Err(e) => {
                    println!("Error: {}", e);
                    process::exit(2)
                }
            }
        }
    };

    let input = std::io::stdin();
    let output = std::io::stdout();

    let mut fish = fish::Interpreter::new(input, output);

    if let Some(strings) = matches.values_of("string") {
        for c in strings {
            fish.push_str(c);
        }
    }

    if let Some(numbers) = matches.values_of("value") {
        for c in numbers {
            let n = match c.parse::<i64>() {
                Ok(v) => v,
                Err(e) => {
                    println!("Error: {}", e);
                    process::exit(2)
                }
            };
            fish.push_i64(n);
        }
    }

    fish.trace = matches.is_present("debug");

    if fish.run(&code_box).is_err() {
        println!("something smells fishy...");
        process::exit(3);
    }

    println!();
}
