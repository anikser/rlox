pub mod common;
pub mod vm;

use common::chunk::*;
use vm::{InterpretError, VM};

fn main() -> Result<(), InterpretError> {
    let mut vm = VM::init();
    let mut chunk = Chunk::new();

    let const_idx = chunk.add_constant(Value(1.2));
    chunk.add_code_op(OpCode::ConstantLong, 1);
    chunk.add_code_constant_long(const_idx, 1);
    chunk.add_code_op(OpCode::Negate, 2);
    chunk.add_code_op(OpCode::Return, 0);

    // print!("{}", chunk);
    vm.interpret(&chunk)?;

    Ok(())
}
