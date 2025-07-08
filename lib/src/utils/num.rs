use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Clone, Copy)]
pub enum Integer {
    U8(u8),
    I16(i16),
    I32(i32),
}

impl Debug for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(arg0) => write!(f, "{arg0}_u8"),
            Self::I16(arg0) => write!(f, "{arg0}_i16"),
            Self::I32(arg0) => write!(f, "{arg0}_i32"),
        }
    }
}

impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(arg0) => write!(f, "{arg0}"),
            Self::I16(arg0) => write!(f, "{arg0}"),
            Self::I32(arg0) => write!(f, "{arg0}"),
        }
    }
}

impl From<u8> for Integer {
    fn from(value: u8) -> Self {
        Integer::U8(value)
    }
}

impl From<i16> for Integer {
    fn from(value: i16) -> Self {
        Integer::I16(value)
    }
}

impl From<i32> for Integer {
    fn from(value: i32) -> Self {
        Integer::I32(value)
    }
}

impl Add for Integer {
    type Output = Integer;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(b1), Integer::U8(b2)) => Integer::U8(b1 + b2),
            (Integer::U8(b1), Integer::I16(s2)) => Integer::I16(b1 as i16 + s2),
            (Integer::U8(b1), Integer::I32(i2)) => Integer::I32(b1 as i32 + i2),
            (Integer::I16(s1), Integer::U8(b2)) => Integer::I16(s1 + b2 as i16),
            (Integer::I16(s1), Integer::I16(s2)) => Integer::I16(s1 + s2),
            (Integer::I16(s1), Integer::I32(i2)) => Integer::I32(s1 as i32 + i2),
            (Integer::I32(i1), Integer::U8(b2)) => Integer::I32(i1 + b2 as i32),
            (Integer::I32(i1), Integer::I16(s2)) => Integer::I32(i1 + s2 as i32),
            (Integer::I32(i1), Integer::I32(i2)) => Integer::I32(i1 + i2),
        }
    }
}

macro_rules! ops {
    ($($type_name: ident [$($num_ty: ty),*] $op_traits: tt, $assign_traits: tt),*) => {
        $(
            $(
                ops!{inner $type_name $num_ty, $op_traits}
                ops!{assign $type_name $num_ty, $assign_traits}
            )*
        )*
    };
    (inner $type_name: ident $num_ty: ty, ($(($op_trait: ident, $func_name: ident)),*)) => {
        $(impl $op_trait<$num_ty> for $type_name {
            type Output = $type_name;

            fn $func_name(self, rhs: $num_ty) -> Self::Output {
                self.$func_name($type_name::from(rhs))
            }
        })*
    };
    (assign $type_name: ident $num_ty: ty, ($(($op_trait: ident, $func_name: ident)),*)) => {
        $(impl $op_trait<$num_ty> for $type_name {
            fn $func_name(&mut self, rhs: $num_ty) {
                self.$func_name($type_name::from(rhs));
            }
        })*
    };
}

ops! {
    Integer [u8, i16, i32] ((Add, add), (Sub, sub), (Mul, mul), (Div, div)), ((AddAssign, add_assign), (SubAssign, sub_assign), (MulAssign, mul_assign), (DivAssign, div_assign)),
    Float [u8, i16, i32, f32] ((Add, add), (Sub, sub), (Mul, mul), (Div, div)), ((AddAssign, add_assign), (SubAssign, sub_assign), (MulAssign, mul_assign), (DivAssign, div_assign))
}

impl AddAssign for Integer {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Integer {
    type Output = Integer;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(b1), Integer::U8(b2)) => Integer::U8(b1 - b2),
            (Integer::U8(b1), Integer::I16(s2)) => Integer::I16(b1 as i16 - s2),
            (Integer::U8(b1), Integer::I32(i2)) => Integer::I32(b1 as i32 - i2),
            (Integer::I16(s1), Integer::U8(b2)) => Integer::I16(s1 - b2 as i16),
            (Integer::I16(s1), Integer::I16(s2)) => Integer::I16(s1 - s2),
            (Integer::I16(s1), Integer::I32(i2)) => Integer::I32(s1 as i32 - i2),
            (Integer::I32(i1), Integer::U8(b2)) => Integer::I32(i1 - b2 as i32),
            (Integer::I32(i1), Integer::I16(s2)) => Integer::I32(i1 - s2 as i32),
            (Integer::I32(i1), Integer::I32(i2)) => Integer::I32(i1 - i2),
        }
    }
}

impl SubAssign for Integer {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Integer {
    type Output = Integer;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(b1), Integer::U8(b2)) => Integer::U8(b1 * b2),
            (Integer::U8(b1), Integer::I16(s2)) => Integer::I16(b1 as i16 * s2),
            (Integer::U8(b1), Integer::I32(i2)) => Integer::I32(b1 as i32 * i2),
            (Integer::I16(s1), Integer::U8(b2)) => Integer::I16(s1 * b2 as i16),
            (Integer::I16(s1), Integer::I16(s2)) => Integer::I16(s1 * s2),
            (Integer::I16(s1), Integer::I32(i2)) => Integer::I32(s1 as i32 * i2),
            (Integer::I32(i1), Integer::U8(b2)) => Integer::I32(i1 * b2 as i32),
            (Integer::I32(i1), Integer::I16(s2)) => Integer::I32(i1 * s2 as i32),
            (Integer::I32(i1), Integer::I32(i2)) => Integer::I32(i1 * i2),
        }
    }
}

impl MulAssign for Integer {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Div for Integer {
    type Output = Integer;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(b1), Integer::U8(b2)) => Integer::U8(b1 / b2),
            (Integer::U8(b1), Integer::I16(s2)) => Integer::I16(b1 as i16 / s2),
            (Integer::U8(b1), Integer::I32(i2)) => Integer::I32(b1 as i32 / i2),
            (Integer::I16(s1), Integer::U8(b2)) => Integer::I16(s1 / b2 as i16),
            (Integer::I16(s1), Integer::I16(s2)) => Integer::I16(s1 / s2),
            (Integer::I16(s1), Integer::I32(i2)) => Integer::I32(s1 as i32 / i2),
            (Integer::I32(i1), Integer::U8(b2)) => Integer::I32(i1 / b2 as i32),
            (Integer::I32(i1), Integer::I16(s2)) => Integer::I32(i1 / s2 as i32),
            (Integer::I32(i1), Integer::I32(i2)) => Integer::I32(i1 / i2),
        }
    }
}

impl DivAssign for Integer {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}


#[derive(Clone, Copy)]
pub enum Float {
    U8(u8),
    I16(i16),
    I32(i32),
    F32(f32),
}

impl Float {
    pub fn as_f32(&self) -> f32 {
        match *self {
            Float::U8(b) => b as f32,
            Float::I16(s) => s as f32,
            Float::I32(i) => i as f32,
            Float::F32(f) => f,
        }
    }
}

impl Debug for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(arg0) => write!(f, "{arg0}_u8"),
            Self::I16(arg0) => write!(f, "{arg0}_i16"),
            Self::I32(arg0) => write!(f, "{arg0}_i32"),
            Self::F32(arg0) => write!(f, "{arg0}_f32"),
        }
    }
}

impl Display for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(arg0) => write!(f, "{arg0}"),
            Self::I16(arg0) => write!(f, "{arg0}"),
            Self::I32(arg0) => write!(f, "{arg0}"),
            Self::F32(arg0) => write!(f, "{arg0}"),
        }
    }
}

impl From<u8> for Float {
    fn from(value: u8) -> Self {
        Float::U8(value)
    }
}

impl From<i16> for Float {
    fn from(value: i16) -> Self {
        Float::I16(value)
    }
}

impl From<i32> for Float {
    fn from(value: i32) -> Self {
        Float::I32(value)
    }
}

impl From<f32> for Float {
    fn from(value: f32) -> Self {
        Float::F32(value)
    }
}

impl From<Integer> for Float {
    fn from(value: Integer) -> Self {
        match value {
            Integer::U8(b) => Float::U8(b),
            Integer::I16(s) => Float::I16(s),
            Integer::I32(i) => Float::I32(i),
        }
    }
}

impl Add for Float {
    type Output = Float;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Float::U8(b1), Float::U8(b2)) => Float::U8(b1 + b2),
            (Float::U8(b1), Float::I16(s2)) => Float::I16(b1 as i16 + s2),
            (Float::U8(b1), Float::I32(i2)) => Float::I32(b1 as i32 + i2),
            (Float::U8(b1), Float::F32(f2)) => Float::F32(b1 as f32 + f2),
            (Float::I16(s1), Float::U8(b2)) => Float::I16(s1 + b2 as i16),
            (Float::I16(s1), Float::I16(s2)) => Float::I16(s1 + s2),
            (Float::I16(s1), Float::I32(i2)) => Float::I32(s1 as i32 + i2),
            (Float::I16(s1), Float::F32(f2)) => Float::F32(s1 as f32 + f2),
            (Float::I32(i1), Float::U8(b2)) => Float::I32(i1 + b2 as i32),
            (Float::I32(i1), Float::I16(s2)) => Float::I32(i1 + s2 as i32),
            (Float::I32(i1), Float::I32(i2)) => Float::I32(i1 + i2),
            (Float::I32(i1), Float::F32(f2)) => Float::F32(i1 as f32 + f2),
            (Float::F32(f1), Float::U8(b2)) => Float::F32(f1 + b2 as f32),
            (Float::F32(f1), Float::I16(s2)) => Float::F32(f1 + s2 as f32),
            (Float::F32(f1), Float::I32(i2)) => Float::F32(f1 + i2 as f32),
            (Float::F32(f1), Float::F32(f2)) => Float::F32(f1 + f2),
        }
    }
}

impl AddAssign for Float {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Add<Integer> for Float {
    type Output = Float;

    fn add(self, rhs: Integer) -> Self::Output {
        let rhs: Float = rhs.into();
        self.add(rhs)
    }
}

impl Add<Float> for Integer {
    type Output = Float;

    fn add(self, rhs: Float) -> Self::Output {
        let this: Float = self.into();
        this.add(rhs)
    }
}

impl AddAssign<Integer> for Float {
    fn add_assign(&mut self, rhs: Integer) {
        *self = *self + rhs;
    }
}


impl Sub for Float {
    type Output = Float;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Float::U8(b1), Float::U8(b2)) => Float::U8(b1 - b2),
            (Float::U8(b1), Float::I16(s2)) => Float::I16(b1 as i16 - s2),
            (Float::U8(b1), Float::I32(i2)) => Float::I32(b1 as i32 - i2),
            (Float::U8(b1), Float::F32(f2)) => Float::F32(b1 as f32 - f2),
            (Float::I16(s1), Float::U8(b2)) => Float::I16(s1 - b2 as i16),
            (Float::I16(s1), Float::I16(s2)) => Float::I16(s1 - s2),
            (Float::I16(s1), Float::I32(i2)) => Float::I32(s1 as i32 - i2),
            (Float::I16(s1), Float::F32(f2)) => Float::F32(s1 as f32 - f2),
            (Float::I32(i1), Float::U8(b2)) => Float::I32(i1 - b2 as i32),
            (Float::I32(i1), Float::I16(s2)) => Float::I32(i1 - s2 as i32),
            (Float::I32(i1), Float::I32(i2)) => Float::I32(i1 - i2),
            (Float::I32(i1), Float::F32(f2)) => Float::F32(i1 as f32 - f2),
            (Float::F32(f1), Float::U8(b2)) => Float::F32(f1 - b2 as f32),
            (Float::F32(f1), Float::I16(s2)) => Float::F32(f1 - s2 as f32),
            (Float::F32(f1), Float::I32(i2)) => Float::F32(f1 - i2 as f32),
            (Float::F32(f1), Float::F32(f2)) => Float::F32(f1 - f2),
        }
    }
}

impl SubAssign for Float {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}


impl Sub<Integer> for Float {
    type Output = Float;

    fn sub(self, rhs: Integer) -> Self::Output {
        let rhs: Float = rhs.into();
        self.sub(rhs)
    }
}

impl Sub<Float> for Integer {
    type Output = Float;

    fn sub(self, rhs: Float) -> Self::Output {
        let this: Float = self.into();
        this.sub(rhs)
    }
}

impl SubAssign<Integer> for Float {
    fn sub_assign(&mut self, rhs: Integer) {
        *self = *self - rhs;
    }
}


impl Mul for Float {
    type Output = Float;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Float::U8(b1), Float::U8(b2)) => Float::U8(b1 * b2),
            (Float::U8(b1), Float::I16(s2)) => Float::I16(b1 as i16 * s2),
            (Float::U8(b1), Float::I32(i2)) => Float::I32(b1 as i32 * i2),
            (Float::U8(b1), Float::F32(f2)) => Float::F32(b1 as f32 * f2),
            (Float::I16(s1), Float::U8(b2)) => Float::I16(s1 * b2 as i16),
            (Float::I16(s1), Float::I16(s2)) => Float::I16(s1 * s2),
            (Float::I16(s1), Float::I32(i2)) => Float::I32(s1 as i32 * i2),
            (Float::I16(s1), Float::F32(f2)) => Float::F32(s1 as f32 * f2),
            (Float::I32(i1), Float::U8(b2)) => Float::I32(i1 * b2 as i32),
            (Float::I32(i1), Float::I16(s2)) => Float::I32(i1 * s2 as i32),
            (Float::I32(i1), Float::I32(i2)) => Float::I32(i1 * i2),
            (Float::I32(i1), Float::F32(f2)) => Float::F32(i1 as f32 * f2),
            (Float::F32(f1), Float::U8(b2)) => Float::F32(f1 * b2 as f32),
            (Float::F32(f1), Float::I16(s2)) => Float::F32(f1 * s2 as f32),
            (Float::F32(f1), Float::I32(i2)) => Float::F32(f1 * i2 as f32),
            (Float::F32(f1), Float::F32(f2)) => Float::F32(f1 * f2),
        }
    }
}

impl MulAssign for Float {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}


impl Mul<Integer> for Float {
    type Output = Float;

    fn mul(self, rhs: Integer) -> Self::Output {
        let rhs: Float = rhs.into();
        self.mul(rhs)
    }
}

impl Mul<Float> for Integer {
    type Output = Float;

    fn mul(self, rhs: Float) -> Self::Output {
        let this: Float = self.into();
        this.mul(rhs)
    }
}

impl MulAssign<Integer> for Float {
    fn mul_assign(&mut self, rhs: Integer) {
        *self = *self * rhs;
    }
}


impl Div for Float {
    type Output = Float;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Float::U8(b1), Float::U8(b2)) => Float::U8(b1 / b2),
            (Float::U8(b1), Float::I16(s2)) => Float::I16(b1 as i16 / s2),
            (Float::U8(b1), Float::I32(i2)) => Float::I32(b1 as i32 / i2),
            (Float::U8(b1), Float::F32(f2)) => Float::F32(b1 as f32 / f2),
            (Float::I16(s1), Float::U8(b2)) => Float::I16(s1 / b2 as i16),
            (Float::I16(s1), Float::I16(s2)) => Float::I16(s1 / s2),
            (Float::I16(s1), Float::I32(i2)) => Float::I32(s1 as i32 / i2),
            (Float::I16(s1), Float::F32(f2)) => Float::F32(s1 as f32 / f2),
            (Float::I32(i1), Float::U8(b2)) => Float::I32(i1 / b2 as i32),
            (Float::I32(i1), Float::I16(s2)) => Float::I32(i1 / s2 as i32),
            (Float::I32(i1), Float::I32(i2)) => Float::I32(i1 / i2),
            (Float::I32(i1), Float::F32(f2)) => Float::F32(i1 as f32 / f2),
            (Float::F32(f1), Float::U8(b2)) => Float::F32(f1 / b2 as f32),
            (Float::F32(f1), Float::I16(s2)) => Float::F32(f1 / s2 as f32),
            (Float::F32(f1), Float::I32(i2)) => Float::F32(f1 / i2 as f32),
            (Float::F32(f1), Float::F32(f2)) => Float::F32(f1 / f2),
        }
    }
}

impl DivAssign for Float {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Div<Integer> for Float {
    type Output = Float;

    fn div(self, rhs: Integer) -> Self::Output {
        let rhs: Float = rhs.into();
        self.div(rhs)
    }
}

impl Div<Float> for Integer {
    type Output = Float;

    fn div(self, rhs: Float) -> Self::Output {
        let this: Float = self.into();
        this.div(rhs)
    }
}

impl DivAssign<Integer> for Float {
    fn div_assign(&mut self, rhs: Integer) {
        *self = *self / rhs;
    }
}
