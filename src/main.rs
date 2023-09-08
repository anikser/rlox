pub mod common;
pub mod compiler;
pub mod vm;

use std::env;

use common::{chunk::*, value::Value};
use vm::{InterpretError, VM};

fn main() -> Result<(), InterpretError> {
    let mut vm = VM::init();

    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => vm.repl(),
        2 => vm.run_file(&args[0]),
        _ => {
            println!("Usage: rlox [path]");
            std::process::exit(64);
        }
    }

    let mut chunk = Chunk::new();

    let const_idx = chunk.add_constant(Value(1.2));
    chunk.add_code_op(OpCode::ConstantLong, 1);
    chunk.add_code_constant_long(const_idx, 1);
    chunk.add_code_op(OpCode::Negate, 2);
    chunk.add_code_op(OpCode::Constant, 3);
    chunk.add_code_constant(const_idx, 3);
    chunk.add_code_op(OpCode::Multiply, 4);
    chunk.add_code_op(OpCode::Return, 5);

    // print!("{}", chunk);
    // vm.interpret(&chunk)?;

    Ok(())
}
