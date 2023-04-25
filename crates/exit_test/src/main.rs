use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        println!("sent exit with status {}", args[1]);
        process::exit(args[1].parse::<i32>().unwrap());
    }
}
