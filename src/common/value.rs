use std::fmt;

#[derive(Clone, Debug)]
pub enum Value {
    Double(f64),
    Boolean(bool),
    Obj(HeapValue),
    Nil,
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Double(_) => false,
            Value::Boolean(val) => !val,
            Value::Obj(_) => false,
            Value::Nil => true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum HeapValue {
    String(Box<str>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Double(val) => write!(f, "{}", val),
            Value::Boolean(val) => write!(f, "{}", val),
            Value::Obj(val) => write!(f, "{:?}", val),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl std::ops::Neg for Value {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        match self {
            Value::Double(double) => Value::Double(-double),
            _ => panic!("UNSUPPORTED OPERATION ON THIS TYPE"),
        }
    }
}
