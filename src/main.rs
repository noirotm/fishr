use std::env;
use std::process;

extern crate getopts;
use getopts::Options;

extern crate fish;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn print_version() {
    println!("{} version {}\nCopyright © 2016 - Marc Noirot", NAME, VERSION);
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "code", "string of instructions to execute instead of FILE", "CODE");
    opts.optmulti("s", "string", "push strings onto the stack before execution starts", "STRING");
    opts.optmulti("v", "value", "push numbers onto the stack before execution starts", "NUMBER");
    opts.optopt("t", "tick", "define a delay between the execution of each instruction", "DELAY");
    opts.optflag("a", "always-tick", "make every instruction cause a tick, even whitespace and skipped instructions");
    opts.optflag("d", "debug", "dump interpreter state before executing an instruction");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("V", "version", "print the program version and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("Error: {}\n", e);
            print_usage(&program, opts);
            process::exit(1);
        },
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(1);
    }

    if matches.opt_present("V") {
        print_version();
        process::exit(1);
    }

    let code_box = match matches.opt_str("c") {
        Some(c) => fish::CodeBox::load_from_string(&c),
        _ => {
            let input = if !matches.free.is_empty() {
                matches.free[0].clone()
            } else {
                print_usage(&program, opts);
                process::exit(1);
            };

            match fish::CodeBox::load_from_file(&input) {
                Ok(cb) => cb,
                Err(e) => {
                    println!("Error: {}", e);
                    process::exit(2);
                }
            }
        }
    };

    let input = std::io::stdin();
    let output = std::io::stdout();

    let mut fish = fish::Interpreter::new(input, output);

    if matches.opt_present("d") {
        fish.trace = true;
    }

    if fish.run(&code_box).is_err() {
        println!("something smells fishy...");
        process::exit(3);
    }

    println!();
}
