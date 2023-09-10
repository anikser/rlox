use std::{fmt, vec};

use super::value::Value;

pub struct FmtWriter<W: std::io::Write>(pub W);

impl<W: std::io::Write> std::fmt::Write for FmtWriter<W> {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        self.0.write_all(s.as_bytes()).map_err(|_| std::fmt::Error)
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result<(), std::fmt::Error> {
        self.0.write_fmt(args).map_err(|_| std::fmt::Error)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum OpCode {
    Return = 0,
    Constant = 1,
    ConstantLong = 2,
    Negate = 3,
    Add = 4,
    Subtract = 5,
    Multiply = 6,
    Divide = 7,
}
impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => OpCode::Return,
            1 => OpCode::Constant,
            2 => OpCode::ConstantLong,
            3 => OpCode::Negate,
            4 => OpCode::Add,
            5 => OpCode::Subtract,
            6 => OpCode::Multiply,
            7 => OpCode::Divide,
            unrecognized => panic!("Unrecognized opcode {}", unrecognized),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ConstantIdx(pub u32);

pub struct Chunk {
    pub code: Vec<u8>,
    lines: Vec<u32>,
    pub constants: Vec<Value>,
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = 0;
        while i < self.code.len() {
            i = self.disassemble(f, i)?;
        }
        Ok(())
    }
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: vec![],
            lines: vec![],
            constants: vec![],
        }
    }

    pub fn add_constant(&mut self, constant: Value) -> ConstantIdx {
        self.constants.push(constant);
        return ConstantIdx((self.constants.len() - 1) as u32);
    }

    pub fn add_code_op(&mut self, code: OpCode, line: u32) {
        self.code.push(code as u8);
        self.lines.push(line);
    }

    // TODO: refactor to combine with add_code_contant_long?
    pub fn add_code_constant(&mut self, constant: ConstantIdx, line: u32) {
        assert!(
            constant.0 <= 255,
            "Single operand (short) constant index must be < 256."
        );
        let bytes = constant.0.to_le_bytes();
        self.code.push(bytes[0]);
        for _ in 0..2 {
            self.lines.push(line);
        }
    }

    pub fn add_code_constant_long(&mut self, constant: ConstantIdx, line: u32) {
        let bytes = constant.0.to_le_bytes();
        assert!(
            constant.0 <= 16777216,
            "Double operand (long) constant index must be < 16777216."
        );
        self.code.extend(&bytes[0..3]);
        for _ in 0..5 {
            self.lines.push(line);
        }
    }

    // Outputs the dissassembed instruction at the offset, and returs the offset of the next instruction
    pub fn disassemble<T: std::fmt::Write>(
        &self,
        f: &mut T,
        offset: usize,
    ) -> Result<usize, std::fmt::Error> {
        let mut i = offset;
        write!(f, "{:03x}", i * 2)?;
        if i > 0 && self.lines[i] == self.lines[i - 1] {
            write!(f, "   | ")?;
        } else {
            write!(f, "{:>3} ", i)?;
        }

        let opcode = OpCode::from(self.code[i]);
        write!(f, "{:?}", opcode)?;
        match opcode {
            OpCode::Constant => {
                i += 1;
                let idx = self.code[i];
                let constant = &self.constants[idx as usize];
                write!(f, "     {} '{}'", idx, constant)?;
            }
            OpCode::ConstantLong => {
                i += 1;
                let mut bytes: [u8; 4] = [0; 4];
                bytes[0..3].copy_from_slice(&self.code[i..i + 3]);
                let idx = u32::from_le_bytes(bytes);
                let constant = &self.constants[idx as usize];
                write!(f, "     {} '{}'", idx, constant)?;
                i += 2;
            }
            _ => (),
        }
        write!(f, "\n")?;
        Ok(i + 1)
    }
}
