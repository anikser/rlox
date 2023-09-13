use std::{
    cell::RefCell,
    fs,
    io::{self, BufRead, Write},
    rc::Rc,
};

use crate::common::Value;
use crate::common::{Chunk, OpCode};

use crate::compiler::*;

#[cfg(debug_assertions)]
use crate::common::FmtWriter;

#[derive(Debug, PartialEq)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}
pub struct VM {
    chunk: Rc<RefCell<Chunk>>,
    // TODO: can we make this better?
    ip: *const u8,
    stack: [Value; STACK_MAX],
    // TODO: revisit this too.. is there a point?
    stack_idx: usize,
}
const STACK_MAX: usize = 256;

type BinaryOp<T> = fn(T, T) -> T;

impl VM {
    pub fn init() -> Self {
        let vm = VM {
            chunk: Rc::new(RefCell::new(Chunk::new())),
            ip: std::ptr::null_mut(),
            stack: [Value::Nil; STACK_MAX],
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
                    // TODO: exception handling here
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
        compile(program, self.chunk.clone())?;
        self.ip = &self.chunk.borrow_mut().code[0];
        self.run()
    }

    // pub fn interpret(&mut self, chunk: &'a Chunk) -> Result<(), InterpretError> {
    //     self.chunk = Some(chunk);
    //     self.ip = &chunk.code[0];
    //     self.run()
    // }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            // FIXME: do not unwrap...
            #[cfg(debug_assertions)]
            {
                self.print_stack();

                let chunk = self.chunk.borrow_mut();
                let start_ptr = &chunk.code[0] as *const u8;
                chunk
                    .disassemble(
                        &mut FmtWriter(std::io::stdout()),
                        (self.ip as usize) - (start_ptr as usize),
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
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::ConstantLong => {
                    let constant = self.read_constant_long();
                    self.push(constant);
                }
                OpCode::Negate => {
                    let negated = -self.pop();
                    self.push(negated);
                }
                OpCode::Add => {
                    self.binary_op_double(|a, b| a + b)?;
                }
                OpCode::Subtract => {
                    self.binary_op_double(|a, b| a - b)?;
                }
                OpCode::Multiply => {
                    self.binary_op_double(|a, b| a * b)?;
                }
                OpCode::Divide => {
                    self.binary_op_double(|a, b| a / b)?;
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Boolean(true)),
                OpCode::False => self.push(Value::Boolean(false)),
                OpCode::Not => {
                    let val = self.pop();
                    self.push(Value::Boolean(val.is_falsey()))
                }
                OpCode::Equal => todo!(),
                OpCode::Greater => todo!(),
                OpCode::Less => todo!(),
            }
        }
    }

    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        let ret = unsafe { *self.ip };
        self.ip = unsafe { self.ip.add(1) };
        return ret;
    }

    #[inline(always)]
    fn read_constant(&mut self) -> Value {
        let idx = self.read_byte();
        return self.chunk.borrow().constants[idx as usize];
    }

    #[inline(always)]
    fn read_constant_long(&mut self) -> Value {
        let mut bytes: [u8; 4] = [0; 4];
        unsafe { std::ptr::copy_nonoverlapping(self.ip, &mut bytes as *mut u8, 3) };
        self.ip = unsafe { self.ip.add(3) };
        let idx = u32::from_le_bytes(bytes);
        return self.chunk.borrow().constants[idx as usize];
    }

    #[inline(always)]
    fn reset_stack(&mut self) {
        // self.stack_top = &mut self.stack[0];
        self.stack_idx = 0;
    }

    #[inline(always)]
    pub fn push(&mut self, value: Value) {
        // TODO: Stack bounds checking/resizing
        self.stack[self.stack_idx] = value;
        self.stack_idx += 1;
        // println!("{:?}", self.stack_top);
        // unsafe { *self.stack_top = value };
        // self.stack_top = unsafe { self.stack_top.add(1) };
        // println!("{:?}", self.stack_top);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Value {
        // self.stack_top = unsafe { self.stack_top.sub(1) };
        // let ret = unsafe { *self.stack_top };
        // ret
        self.stack_idx -= 1;
        self.stack[self.stack_idx]
    }

    pub fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack_idx - 1 - distance]
    }

    #[inline(always)]
    fn binary_op_double(&mut self, f: BinaryOp<f64>) -> Result<(), InterpretError> {
        if !matches!(self.peek(0), Value::Double(_)) || !matches!(self.peek(1), Value::Double(_)) {
            return Err(self.runtime_error("Operands must be numbers."));
        }
        let a = self.pop();
        let b = self.pop();
        match a {
            Value::Double(right) => {
                if let Value::Double(left) = b {
                    Ok(self.push(Value::Double(f(left, right))))
                } else {
                    panic!("Found unexpected non-Double value after validation.");
                }
            }
            _ => panic!("Found unexpected non-Double value after validation."),
        }
    }

    #[cfg(debug_assertions)]
    fn print_stack(&mut self) {
        print!("        ");
        for i in 0..self.stack_idx {
            print!("[{}]", self.stack[i]);
        }
        println!();
    }

    fn runtime_error(&self, message: &str) -> InterpretError {
        println!("{}", message);

        let chunk = self.chunk.borrow();
        let start_ptr = &chunk.code[0] as *const u8;
        let offset = (self.ip as usize) - (start_ptr as usize);
        let line = chunk.lines[offset];
        println!("line [{}] in script", line);
        InterpretError::RuntimeError
    }
}
