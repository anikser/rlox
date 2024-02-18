pub mod common;
pub mod compiler;
pub mod vm;

use std::env;

use vm::{InterpretError, VM};

fn main() -> Result<(), InterpretError> {
    let mut vm = VM::init();

    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => vm.repl(),
        2 => vm.run_file(&args[1]),
        _ => {
            println!("Usage: rlox [path]");
            std::process::exit(64);
        }
    }
    Ok(())
}
