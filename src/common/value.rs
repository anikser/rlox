use std::alloc::{alloc, Layout};
use std::fmt;

use super::table::Hashable;

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
            (HeapValue::String(a), HeapValue::String(b)) => a.eq(b),
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
    String(BoxedObjString),
}

#[derive(Clone, Copy, Debug)]
struct Inner {
    length: usize,
    hash: u32,
}

#[derive(Debug)]
pub struct ObjString {
    inner: Inner,
    data: [u8],
}

impl ObjString {
    fn get_layout(length: usize) -> (Layout, usize) {
        Layout::array::<u8>(length)
            .and_then(|layout| Layout::new::<Inner>().extend(layout))
            .unwrap()
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.data[0..self.inner.length]) }
    }

    pub fn len(&self) -> usize {
        self.inner.length
    }

    fn calc_hash(key: &str) -> u32 {
        let mut hash: u32 = 2166136261;
        for x in key.as_bytes() {
            hash ^= *x as u32;
            (hash, _) = hash.overflowing_mul(16777619);
        }
        hash
    }
}

#[derive(Debug)]
pub struct BoxedObjString(Box<ObjString>);

impl BoxedObjString {
    pub fn of_ref(source: &String) -> Self {
        let byte_array = source.as_bytes();
        let inner = Inner {
            length: byte_array.len(),
            hash: ObjString::calc_hash(source.as_str()),
        };
        let (layout, arr_base) = ObjString::get_layout(inner.length);
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            panic!("Failed to allocate ObjString");
        }
        unsafe {
            ptr.cast::<Inner>().write(inner);
            let tmp_ptr = ptr.cast::<u8>().add(arr_base);
            for (i, x) in byte_array.iter().enumerate() {
                tmp_ptr.add(i).write(*x);
            }
        }
        unsafe {
            Self(Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(ptr as *mut usize, layout.size())
                    as *mut ObjString,
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn of(source: String) -> Self {
        Self::of_ref(&source)
    }
}

impl Clone for BoxedObjString {
    fn clone(&self) -> Self {
        let (layout, arr_base) = Layout::array::<u8>(self.0.inner.length)
            .and_then(|layout| Layout::new::<Inner>().extend(layout))
            .unwrap();
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            panic!("Failed to allocate ObjString");
        }
        unsafe {
            ptr.cast::<Inner>().write(self.0.inner);
            let tmp_ptr = ptr.cast::<u8>().add(arr_base);
            for i in 0..(self.0.inner.length) {
                tmp_ptr.add(i).write(self.0.data[i]);
            }
        }
        unsafe {
            Self(Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(ptr as *mut usize, layout.size())
                    as *mut ObjString,
            ))
        }
    }
}

impl Hashable for BoxedObjString {
    #[inline(always)]
    fn hash(&self) -> u32 {
        self.0.inner.hash
    }
}

impl PartialEq for BoxedObjString {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            self.0.inner.length == other.0.inner.length
                && std::slice::from_raw_parts(self.0.data.as_ptr(), self.0.inner.length).eq(
                    std::slice::from_raw_parts(other.0.data.as_ptr(), other.0.inner.length),
                )
        }
    }
}

impl fmt::Display for HeapValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeapValue::String(s) => {
                write!(f, "{}", s.as_str())
            }
        }
    }
}
