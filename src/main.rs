use std::env;
use std::process;

extern crate fish;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <fish script>\n", args[0]);
        process::exit(1);
    }

    // let codebox = fish::CodeBox::load_from_string("2*n;");

    let codebox = match fish::CodeBox::load_from_file(&args[1]) {
        Ok(cb) => cb,
        Err(e) => {
            println!("Error: {}", e);
            process::exit(2);
        }
    };

    let input = std::io::stdin();
    let output = std::io::stdout();

    let mut fish = fish::Interpreter::new(input, output);

    let result = fish.run(&codebox);
    if result.is_err() {
        println!("something smells fishy...")
    }
}
