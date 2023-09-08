use std::{
    fs,
    io::{self, BufRead, Write},
};

use crate::common::chunk::{Chunk, OpCode};
use crate::common::value::Value;

#[cfg(debug_assertions)]
use crate::common::chunk::FmtWriter;

#[derive(Debug, PartialEq)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}
pub struct VM<'a> {
    chunk: Option<&'a Chunk>,
    // TODO: can we make this better?
    ip: *const u8,
    stack: [Value; STACK_MAX],
    // TODO: revisit this too.. is there a point?
    stack_idx: usize,
}
const STACK_MAX: usize = 256;

type BinaryOp = fn(Value, Value) -> Value;

impl<'a> VM<'a> {
    pub fn init() -> Self {
        let vm = VM {
            chunk: None,
            ip: std::ptr::null_mut(),
            stack: [Value(0.0); STACK_MAX],
            // stack_top: std::ptr::null_mut(),
            stack_idx: 0,
        };
        vm
    }
    pub fn repl(&mut self) {
        let stdin = io::stdin();
        let mut iterator = stdin.lock().lines();
        loop {
            // let mut buffer = String::new();
            print!(">");
            io::stdout().flush().unwrap();
            // let result = { stdin.read_line(&mut buffer) };
            match iterator.next().unwrap() {
                // Ok(bytes_read) if bytes_read == 0 => {
                Ok(line) if line.len() == 0 => {
                    println!();
                    break;
                }
                Ok(line) => {
                    self.interpret(line);
                }
                Err(err) => {
                    println!("{}", err);
                    break;
                }
            }
        }
    }

    pub fn run_file(&mut self, path: &String) {
        let contents = fs::read_to_string(path);
        match contents {
            Ok(contents) => {
                match self.interpret(contents) {
                    Ok(()) => (),
                    Err(InterpretError::CompileError) => std::process::exit(65),
                    Err(InterpretError::RuntimeError) => std::process::exit(70),
                };
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => println!("File not found."),
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                println!("Permission denied reading file.")
            }
            Err(_) => println!("Unexpected error reading file."),
        }
    }

    pub fn interpret(&mut self, program: String) -> Result<(), InterpretError> {
        let chunk = crate::compiler::compile(program);
        Ok(())
    }

    // pub fn interpret(&mut self, chunk: &'a Chunk) -> Result<(), InterpretError> {
    //     self.chunk = Some(chunk);
    //     self.ip = &chunk.code[0];
    //     self.run()
    // }

    fn run(&mut self) -> Result<(), InterpretError> {
        match self.chunk {
            None => Err(InterpretError::CompileError),
            Some(chunk) => loop {
                // FIXME: do not unwrap...
                #[cfg(debug_assertions)]
                {
                    self.print_stack();

                    chunk
                        .disassemble(
                            &mut FmtWriter(std::io::stdout()),
                            (self.ip as usize) - (&chunk.code[0] as *const u8 as usize),
                        )
                        .unwrap();
                }

                let opcode = OpCode::from(self.read_byte());
                match opcode {
                    OpCode::Return => {
                        println!("{}", self.pop());
                        return Ok(());
                    }
                    OpCode::Constant => {
                        let constant = self.read_constant(&chunk);
                        self.push(constant);
                    }
                    OpCode::ConstantLong => {
                        let constant = self.read_constant_long(&chunk);
                        self.push(constant);
                    }
                    OpCode::Negate => {
                        let negated = -self.pop();
                        self.push(negated);
                    }
                    OpCode::Add => {
                        self.binary_op(|a, b| a + b);
                    }
                    OpCode::Subtract => {
                        self.binary_op(|a, b| a - b);
                    }
                    OpCode::Multiply => {
                        self.binary_op(|a, b| a * b);
                    }
                    OpCode::Divide => {
                        self.binary_op(|a, b| a / b);
                    }
                }
            },
        }
    }

    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        let ret = unsafe { *self.ip };
        self.ip = unsafe { self.ip.add(1) };
        return ret;
    }

    #[inline(always)]
    fn read_constant(&mut self, chunk: &Chunk) -> Value {
        let idx = self.read_byte();
        return chunk.constants[idx as usize];
    }

    #[inline(always)]
    fn read_constant_long(&mut self, chunk: &Chunk) -> Value {
        let mut bytes: [u8; 4] = [0; 4];
        unsafe { std::ptr::copy_nonoverlapping(self.ip, &mut bytes as *mut u8, 3) };
        self.ip = unsafe { self.ip.add(3) };
        let idx = u32::from_le_bytes(bytes);
        return chunk.constants[idx as usize];
    }

    #[inline(always)]
    fn reset_stack(&mut self) {
        // self.stack_top = &mut self.stack[0];
        self.stack_idx = 0;
    }

    pub fn push(&mut self, value: Value) {
        // TODO: Stack bounds checking/resizing
        self.stack[self.stack_idx] = value;
        self.stack_idx += 1;
        // println!("{:?}", self.stack_top);
        // unsafe { *self.stack_top = value };
        // self.stack_top = unsafe { self.stack_top.add(1) };
        // println!("{:?}", self.stack_top);
    }

    pub fn pop(&mut self) -> Value {
        // self.stack_top = unsafe { self.stack_top.sub(1) };
        // let ret = unsafe { *self.stack_top };
        // ret
        self.stack_idx -= 1;
        self.stack[self.stack_idx]
    }

    fn binary_op(&mut self, f: BinaryOp) {
        let a = self.pop();
        let b = self.pop();
        self.push(f(a, b));
    }

    #[cfg(debug_assertions)]
    fn print_stack(&mut self) {
        print!("        ");
        for i in 0..self.stack_idx {
            print!("[{}]", self.stack[i]);
        }
        // let mut sp = &self.stack[0] as *const Value;
        // println!("{:?}, {:?}", self.stack_top, sp);
        // while sp < self.stack_top {
        //     print!("[{}]", unsafe { *sp });
        //     sp = unsafe { sp.add(1) };
        // }
        println!();
    }
}
