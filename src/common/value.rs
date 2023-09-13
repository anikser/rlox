use std::fmt;

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Double(f64),
    Boolean(bool),
    Nil,
}
impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Double(_) => false,
            Value::Boolean(val) => !val,
            Value::Nil => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Double(val) => write!(f, "{}", val),
            Value::Boolean(val) => write!(f, "{}", val),
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

impl std::ops::Add for Value {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Value::Double(double) => {
                if let Value::Double(rhs_double) = rhs {
                    Value::Double(double + rhs_double)
                } else {
                    panic!("BOTH OPERANDS OF ADD MUST BE DOUBLE")
                }
            }
            _ => panic!("UNSUPPORTED OPERATION ON THIS TYPE"),
        }
    }
}
