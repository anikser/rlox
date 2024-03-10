use std::fmt;

#[derive(Clone, Debug)]
pub enum Value {
    Double(f64),
    Boolean(bool),
    Object(Obj),
    Nil,
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Double(_) => false,
            Value::Boolean(val) => !val,
            Value::Object(_) => false,
            Value::Nil => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Double(val) => write!(f, "{}", val),
            Value::Boolean(val) => write!(f, "{}", val),
            Value::Object(val) => write!(f, "{}", val),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Double(a), Self::Double(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => a == b,
            (Self::Nil, Self::Nil) => true,
            _ => false,
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

#[derive(Clone, Debug)]
pub struct Obj {
    pub value: HeapValue,
    pub next: Option<*const Obj>,
}

impl PartialEq for Obj {
    fn eq(&self, other: &Obj) -> bool {
        match (&self.value, &other.value) {
            (HeapValue::String(a), HeapValue::String(b)) => a.eq(&b),
        }
    }
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub enum HeapValue {
    String(ObjString),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjString {
    pub value: Box<str>,
    pub hash: u32,
}
impl ObjString {
    pub fn of_borrow(source: &String) -> Self {
        Self {
            value: source.as_str().into(),
            hash: Self::hash(source.as_str()),
        }
    }

    pub fn of(source: String) -> Self {
        Self {
            value: source.as_str().into(),
            hash: Self::hash(source.as_str()),
        }
    }

    fn hash(key: &str) -> u32 {
        let mut hash: u32 = 2166136261;
        for x in key.as_bytes() {
            hash ^= *x as u32;
            (hash, _) = hash.overflowing_mul(16777619);
        }
        return hash;
    }
}

impl fmt::Display for HeapValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeapValue::String(s) => write!(f, "{}", s.value),
        }
    }
}
