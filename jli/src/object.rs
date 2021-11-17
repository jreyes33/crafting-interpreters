use crate::expr::VisitorResult;
use crate::Result;
use std::any::Any;
use std::fmt;
use std::rc::Rc;

const OPS_NUMBERS: &str = "Operands must be numbers.";
const OPS_ADD: &str = "Operands must be two numbers or two strings.";

pub trait Object: fmt::Display + ObjectEq {
    fn truthy(&self) -> bool {
        true
    }

    fn try_neg(&self) -> Result<f64> {
        Err("Operand must be a number.".into())
    }

    fn try_add(&self, _other: &dyn Object) -> VisitorResult {
        Err(OPS_ADD.into())
    }

    fn try_gt(&self, _other: &dyn Object) -> Result<bool> {
        Err(OPS_NUMBERS.into())
    }

    fn try_ge(&self, _other: &dyn Object) -> Result<bool> {
        Err(OPS_NUMBERS.into())
    }

    fn try_lt(&self, _other: &dyn Object) -> Result<bool> {
        Err(OPS_NUMBERS.into())
    }

    fn try_le(&self, _other: &dyn Object) -> Result<bool> {
        Err(OPS_NUMBERS.into())
    }

    fn try_sub(&self, _other: &dyn Object) -> Result<f64> {
        Err(OPS_NUMBERS.into())
    }

    fn try_div(&self, _other: &dyn Object) -> Result<f64> {
        Err(OPS_NUMBERS.into())
    }

    fn try_mul(&self, _other: &dyn Object) -> Result<f64> {
        Err(OPS_NUMBERS.into())
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub trait ObjectEq: AsAny {
    fn equal(&self, other: &dyn Object) -> bool;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Based on this answer: https://stackoverflow.com/a/25359060
impl<T: PartialEq + Any> ObjectEq for T {
    fn equal(&self, other: &dyn Object) -> bool {
        if let Some(o) = other.as_any().downcast_ref() {
            return self == o;
        }
        false
    }
}

impl Object for f64 {
    fn try_neg(&self) -> Result<f64> {
        Ok(-self)
    }

    fn try_add(&self, other: &dyn Object) -> VisitorResult {
        if let Some(o) = other.as_any().downcast_ref() {
            return Ok(Rc::new(self + o));
        }
        Err(OPS_ADD.into())
    }

    fn try_gt(&self, other: &dyn Object) -> Result<bool> {
        if let Some(o) = other.as_any().downcast_ref() {
            return Ok(self > o);
        }
        Err(OPS_NUMBERS.into())
    }

    fn try_ge(&self, other: &dyn Object) -> Result<bool> {
        if let Some(o) = other.as_any().downcast_ref() {
            return Ok(self >= o);
        }
        Err(OPS_NUMBERS.into())
    }

    fn try_lt(&self, other: &dyn Object) -> Result<bool> {
        if let Some(o) = other.as_any().downcast_ref() {
            return Ok(self < o);
        }
        Err(OPS_NUMBERS.into())
    }

    fn try_le(&self, other: &dyn Object) -> Result<bool> {
        if let Some(o) = other.as_any().downcast_ref() {
            return Ok(self <= o);
        }
        Err(OPS_NUMBERS.into())
    }

    fn try_sub(&self, other: &dyn Object) -> Result<f64> {
        if let Some(o) = other.as_any().downcast_ref() {
            return Ok(self - o);
        }
        Err(OPS_NUMBERS.into())
    }

    fn try_div(&self, other: &dyn Object) -> Result<f64> {
        if let Some(o) = other.as_any().downcast_ref() {
            return Ok(self / o);
        }
        Err(OPS_NUMBERS.into())
    }

    fn try_mul(&self, other: &dyn Object) -> Result<f64> {
        if let Some(o) = other.as_any().downcast_ref() {
            return Ok(self * o);
        }
        Err(OPS_NUMBERS.into())
    }
}

impl Object for bool {
    fn truthy(&self) -> bool {
        *self
    }
}

impl Object for String {
    fn try_add(&self, other: &dyn Object) -> VisitorResult {
        if let Some(o) = other.as_any().downcast_ref::<String>() {
            return Ok(Rc::new(self.to_string() + o));
        }
        Err(OPS_ADD.into())
    }
}

impl Object for Nil {
    fn truthy(&self) -> bool {
        false
    }
}

#[derive(PartialEq)]
pub struct Nil;

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nil")
    }
}
