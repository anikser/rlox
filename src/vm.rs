use std::{
    cell::RefCell,
    fs,
    io::{self, BufRead, Write},
    rc::Rc,
};

use crate::common::Table;
use crate::common::{Chunk, ObjectRef, OpCode, ObjectHeap, ObjectData};
use crate::common::Value;

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
    heap: ObjectHeap,
    strings: Table<ObjectRef, ()>,
}

const STACK_MAX: usize = 256;

type BinaryOp<I, O> = fn(I, I) -> O;

impl VM {
    pub fn init() -> Self {
        
        VM {
            chunk: Rc::new(RefCell::new(Chunk::new())),
            ip: std::ptr::null_mut(),
            stack: std::array::from_fn(|_| Value::Nil),
            // stack_top: std::ptr::null_mut(),
            stack_idx: 0,
            heap: ObjectHeap::new(),
            strings: Table::new(),
        }
    }
    pub fn repl(&mut self) {
        let stdin = io::stdin();
        let mut iterator = stdin.lock().lines();
        loop {
            // let mut buffer = String::new();
            print!(">");
            io::stdout().flush().unwrap();
            // let result = { stdin.read_line(&mut buffer) };
            match iterator.next() {
                // Ok(bytes_read) if bytes_read == 0 => {
                Some(Ok(line)) if line.is_empty() => {
                    println!();
                    break;
                }
                Some(Ok(line)) => {
                    // TODO: exception handling here
                    self.interpret(line)
                        .unwrap_or_else(|_| println!("Error executing REPL"));
                }
                Some(Err(err)) => {
                    println!("{}", err);
                    break;
                }
                None => {
                    break;
                }
            }
        }
    }

    pub fn run_file(&mut self, path: &String) {
        println!("Readind path {}", path);
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
            Err(e) => println!("Unexpected error reading file. {:?}", e),
        }
    }

    pub fn interpret(&mut self, program: String) -> Result<(), InterpretError> {
        compile(program, self.chunk.clone(), &self.heap)?;
        self.ip = &self.chunk.borrow_mut().code[0];
        self.run()
    }

    // pub fn interpret(&mut self, chunk: &'a Chunk) -> Result<(), InterpretError> {
    //     self.chunk = Some(chunk);
    //     self.ip = &chunk.code[0];
    //     self.run()
    // }

    fn run(&mut self) -> Result<(), InterpretError> {
        // Manage GC roots before starting execution
        self.manage_stack_roots();
        
        let result = self.run_inner();
        
        // Clear roots after execution
        self.clear_stack_roots();
        
        result
    }
    
    fn run_inner(&mut self) -> Result<(), InterpretError> {
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
                    let val = self.pop();
                    match val {
                        Value::Object(obj_ref) => {
                            if let Some(obj) = self.heap.get(obj_ref) {
                                println!("{}", obj.data);
                            } else {
                                println!("[invalid object]");
                            }
                        }
                        _ => println!("{}", val),
                    }
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
                    if self.is_string(0) && self.is_string(1) {
                        self.string_concat()?;
                    } else {
                        self.binary_op(|a, b| Value::Double(a + b))?;
                    }
                }
                OpCode::Subtract => self.binary_op(|a, b| Value::Double(a - b))?,
                OpCode::Multiply => self.binary_op(|a, b| Value::Double(a * b))?,
                OpCode::Divide => self.binary_op(|a, b| Value::Double(a / b))?,
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Boolean(true)),
                OpCode::False => self.push(Value::Boolean(false)),
                OpCode::Not => {
                    let val = self.pop();
                    self.push(Value::Boolean(val.is_falsey()))
                }
                OpCode::Equal => {
                    let a = self.pop();
                    let b = self.pop();

                    self.push(Value::Boolean(a == b))
                }
                OpCode::Greater => self.binary_op(|a, b| Value::Boolean(a > b))?,
                OpCode::Less => self.binary_op(|a, b| Value::Boolean(a < b))?,
            }
        }
    }

    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        let ret = unsafe { *self.ip };
        self.ip = unsafe { self.ip.add(1) };
        ret
    }

    #[inline(always)]
    fn read_constant(&mut self) -> Value {
        let idx = self.read_byte();
        let val = self.chunk.borrow().constants[idx as usize].clone();
        val
    }

    #[inline(always)]
    fn read_constant_long(&mut self) -> Value {
        let mut bytes: [u8; 4] = [0; 4];
        unsafe { std::ptr::copy_nonoverlapping(self.ip, &mut bytes as *mut u8, 3) };
        self.ip = unsafe { self.ip.add(3) };
        let idx = u32::from_le_bytes(bytes);
        return self.chunk.borrow().constants[idx as usize].clone();
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
        self.stack[self.stack_idx].clone()
    }

    #[inline(always)]
    pub fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack_idx - 1 - distance]
    }

    #[inline(always)]
    fn binary_op(&mut self, f: BinaryOp<f64, Value>) -> Result<(), InterpretError> {
        if !matches!(self.peek(0), Value::Double(_)) || !matches!(self.peek(1), Value::Double(_)) {
            return Err(self.runtime_error("Operands must be numbers."));
        }
        let a = self.pop();
        let b = self.pop();
        match (a, b) {
            (Value::Double(right), Value::Double(left)) => {
                self.push(f(left, right));
                Ok(())
            },
            _ => panic!("Found unexpected non-Double value after validation."),
        }
    }

    fn string_concat(&mut self) -> Result<(), InterpretError> {
        if !self.is_string(0) || !self.is_string(1) {
            return Err(self.runtime_error("Operands must be strings."));
        }
        let a = self.pop();
        let b = self.pop();
        match (a, b) {
            (Value::Object(right_ref), Value::Object(left_ref)) => {
                let left_str = self.heap.get_string(left_ref).unwrap();
                let right_str = self.heap.get_string(right_ref).unwrap();
                
                let mut new_string = String::with_capacity(left_str.len() + right_str.len());
                new_string.push_str(&left_str);
                new_string.push_str(&right_str);
                
                let result_ref = self.heap.alloc_string(&new_string);
                self.push(Value::Object(result_ref));
                Ok(())
            }
            _ => panic!("Found unexpected non-string value after validation."),
        }
    }
    
    fn is_string(&self, distance: usize) -> bool {
        match self.peek(distance) {
            Value::Object(obj_ref) => {
                self.heap.get(*obj_ref).map_or(false, |obj| {
                    matches!(obj.data, ObjectData::String(_))
                })
            }
            _ => false,
        }
    }

    #[cfg(debug_assertions)]
    fn print_stack(&mut self) {
        print!("        ");
        for i in 0..self.stack_idx {
            match &self.stack[i] {
                Value::Object(obj_ref) => {
                    if let Some(obj) = self.heap.get(*obj_ref) {
                        print!("[{}]", obj.data);
                    } else {
                        print!("[invalid object]");
                    }
                }
                val => print!("[{}]", val),
            }
        }
        println!();
    }

    fn runtime_error(&mut self, message: &str) -> InterpretError {
        println!("{}", message);

        let chunk = self.chunk.borrow();
        let start_ptr = &chunk.code[0] as *const u8;
        let offset = (self.ip as usize) - (start_ptr as usize);
        let line = chunk.lines[offset];
        println!("line [{}] in script", line);
        InterpretError::RuntimeError
    }

    fn manage_stack_roots(&mut self) {
        // Add all object refs on the stack as GC roots
        for i in 0..self.stack_idx {
            if let Value::Object(obj_ref) = &self.stack[i] {
                self.heap.add_root(*obj_ref);
            }
        }
    }
    
    fn clear_stack_roots(&mut self) {
        // Remove all object refs from GC roots
        for i in 0..self.stack_idx {
            if let Value::Object(obj_ref) = &self.stack[i] {
                self.heap.remove_root(*obj_ref);
            }
        }
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        // Heap will be dropped automatically
    }
}
