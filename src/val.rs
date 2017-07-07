use std::fmt;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Val {
    Byte(u8),
    Int(i64),
    Float(f64),
}

impl Val {
    pub fn to_i64(&self) -> i64 {
        match *self {
            Val::Byte(val) => val as i64,
            Val::Int(val) => val,
            Val::Float(val) => val as i64,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Val::Byte(val) => val,
            Val::Int(val) => val as u8,
            Val::Float(val) => val as u8,
        }
    }

    pub fn to_f64(&self) -> f64 {
        match *self {
            Val::Byte(val) => val as f64,
            Val::Int(val) => val as f64,
            Val::Float(val) => val,
        }
    }

    pub fn checked_add(&self, other: Val) -> Option<Val> {
        match (*self, other) {
            (Val::Float(f), v) => Some(Val::Float(f + v.to_f64())),
            (v, Val::Float(f)) => Some(Val::Float(v.to_f64() + f)),
            _ => self.to_i64().checked_add(other.to_i64()).map(|v| Val::Int(v)),
        }
    }

    pub fn checked_sub(&self, other: Val) -> Option<Val> {
        match (*self, other) {
            (Val::Float(f), v) => Some(Val::Float(f - v.to_f64())),
            (v, Val::Float(f)) => Some(Val::Float(v.to_f64() - f)),
            _ => self.to_i64().checked_sub(other.to_i64()).map(|v| Val::Int(v)),
        }
    }

    pub fn checked_mul(&self, other: Val) -> Option<Val> {
        match (*self, other) {
            (Val::Float(f), v) => Some(Val::Float(f * v.to_f64())),
            (v, Val::Float(f)) => Some(Val::Float(v.to_f64() * f)),
            _ => self.to_i64().checked_mul(other.to_i64()).map(|v| Val::Int(v)),
        }
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Val::Byte(val) => write!(f, "{}", val),
            Val::Int(val) => write!(f, "{}", val),
            Val::Float(val) => write!(f, "{}", val),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn val_to_u8_works() {
        let val = Val::Byte(15);
        assert_eq!(val.to_u8(), 15);

        let val = Val::Int(54);
        assert_eq!(val.to_u8(), 54);
    }

    #[test]
    fn val_to_i64_works() {
        let val = Val::Byte(15);
        assert_eq!(val.to_i64(), 15);

        let val = Val::Int(54);
        assert_eq!(val.to_i64(), 54)
    }

    #[test]
    fn val_to_f64_works() {
        let val = Val::Byte(15);
        assert_eq!(val.to_f64(), 15.0);
    }
}
