use crate::{common::chunk::Chunk, compiler::scanner::Scanner, vm::InterpretError};

pub fn compile(source: String) -> Result<Chunk, InterpretError> {
    let mut scanner = Scanner::init(source);
    Ok(Chunk::new())
}
