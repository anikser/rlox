use std::{
    cell::RefCell,
    fs,
    io::{self, BufRead, Write},
    rc::Rc,
};

use crate::common::Table;
use crate::common::{BoxedObjString, Chunk, Obj, OpCode};
use crate::common::{HeapValue, Value};

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
    objects: Option<*const Obj>,
    strings: Table<BoxedObjString, ()>,
}

const STACK_MAX: usize = 256;

type BinaryOp<I, O> = fn(I, I) -> O;

impl VM {
    pub fn init() -> Self {
        let vm = VM {
            chunk: Rc::new(RefCell::new(Chunk::new())),
            ip: std::ptr::null_mut(),
            stack: std::array::from_fn(|_| Value::Nil),
            // stack_top: std::ptr::null_mut(),
            stack_idx: 0,
            objects: None,
            strings: Table::new(),
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
            match iterator.next() {
                // Ok(bytes_read) if bytes_read == 0 => {
                Some(Ok(line)) if line.len() == 0 => {
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
                OpCode::Add => match (self.peek(0), self.peek(1)) {
                    (
                        Value::Object(Obj {
                            value: HeapValue::String(_),
                            next: _,
                        }),
                        Value::Object(Obj {
                            value: HeapValue::String(_),
                            next: _,
                        }),
                    ) => {
                        self.string_concat()?;
                    }
                    (_, _) => self.binary_op(|a, b| Value::Double(a + b))?,
                },
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
        return ret;
    }

    #[inline(always)]
    fn read_constant(&mut self) -> Value {
        let idx = self.read_byte();
        let val = self.chunk.borrow().constants[idx as usize].clone();
        return val;
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
            (Value::Double(right), Value::Double(left)) => Ok(self.push(f(left, right))),
            _ => panic!("Found unexpected non-Double value after validation."),
        }
    }

    fn string_concat(&mut self) -> Result<(), InterpretError> {
        if !matches!(
            self.peek(0),
            Value::Object(Obj {
                value: HeapValue::String(_),
                next: _,
            })
        ) || !matches!(
            self.peek(1),
            Value::Object(Obj {
                value: HeapValue::String(_),
                next: _,
            })
        ) {
            return Err(self.runtime_error("Operands must be strings."));
        }
        let a = self.pop();
        let b = self.pop();
        match (a, b) {
            (
                Value::Object(Obj {
                    value: HeapValue::String(right),
                    next: _,
                }),
                Value::Object(Obj {
                    value: HeapValue::String(left),
                    next: _,
                }),
            ) => {
                let mut new_string = String::with_capacity(left.len() + right.len());
                let left_str = left.as_str();
                let right_str = right.as_str();
                new_string.push_str(left_str);
                new_string.push_str(right_str);

                let res = BoxedObjString::of(new_string);

                let obj = self.create_object(HeapValue::String(res));
                self.push(Value::Object(obj));
                Ok(())
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

    fn runtime_error(&mut self, message: &str) -> InterpretError {
        println!("{}", message);

        let chunk = self.chunk.borrow();
        let start_ptr = &chunk.code[0] as *const u8;
        let offset = (self.ip as usize) - (start_ptr as usize);
        let line = chunk.lines[offset];
        println!("line [{}] in script", line);
        InterpretError::RuntimeError
    }

    fn create_object(&mut self, value: HeapValue) -> Obj {
        let obj = Obj {
            value,
            next: self.objects.take(),
        };
        self.objects = Some(&obj);
        obj
    }

    fn free_objects(&mut self) {
        let maybe_obj = self.objects;
        while let Some(obj_ref) = maybe_obj {
            unsafe {
                let obj = obj_ref.as_ref();
            };
        }
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free_objects();
    }
}
