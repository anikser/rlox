use std::fmt;

use super::heap::ObjectRef;

#[derive(Clone, Debug)]
pub enum Value {
    Double(f64),
    Boolean(bool),
    Object(ObjectRef),
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
            Value::Object(obj_ref) => write!(f, "[object@{}]", obj_ref.0),
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

// Old heap types removed - now using ObjectRef from heap module
