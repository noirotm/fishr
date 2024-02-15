use clap::Parser;
use std::path::PathBuf;
use std::{process, time::Duration};

#[derive(Parser)]
#[command(version, author = "marc.noirot@gmail.com", about, long_about = None)]
struct Args {
    /// set the input file to use
    #[arg(value_name = "FILE", conflicts_with = "code")]
    input: Option<PathBuf>,

    /// string of instructions to execute instead of FILE
    #[arg(short = 'c', long = "code")]
    code: Option<String>,

    /// push strings onto the stack before execution starts
    #[arg(short = 's', long = "string")]
    strings: Vec<String>,

    /// push numbers onto the stack before execution starts
    #[arg(short = 'v', long = "value")]
    numbers: Vec<i64>,

    /// define a delay between the execution of each instruction
    #[arg(short = 't', long = "tick")]
    tick: Option<u64>,

    /// make every instruction cause a tick, even whitespace and skipped instructions
    #[arg(short = 'a', long = "always-tick")]
    always_tick: bool,

    /// dump interpreter state before executing an instruction
    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    let code_box = match args.code {
        Some(c) => fish::CodeBox::load_from_string(&c),
        None => {
            let input = args.input.unwrap_or_else(|| {
                println!("Error: missing file name");
                process::exit(1)
            });
            fish::CodeBox::load_from_file(&input).unwrap_or_else(|e| {
                println!("Error: {}", e);
                process::exit(2)
            })
        }
    };

    let input = std::io::stdin();
    let output = std::io::stdout();

    let mut fish = fish::Interpreter::new(input, output);

    for s in &args.strings {
        fish.push_str(s);
    }

    for n in args.numbers {
        fish.push_i64(n);
    }

    fish.trace = args.debug;

    if let Some(seconds) = args.tick {
        fish.tick = Some(Duration::from_secs(seconds));
    }

    if fish.run(&code_box).is_err() {
        println!("something smells fishy...");
        process::exit(3);
    }

    println!();
}
