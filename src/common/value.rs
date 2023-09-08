use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct Value(pub f64);

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl std::ops::Neg for Value {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Value(-self.0)
    }
}

impl std::ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Value(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Value(self.0 - rhs.0)
    }
}

impl std::ops::Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Value(self.0 * rhs.0)
    }
}

impl std::ops::Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Value(self.0 / rhs.0)
    }
}
