#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::as_conversions, clippy::cast_ptr_alignment, trivial_casts)]

#[cfg(feature = "alloc")]
extern crate alloc;
pub mod eval;
pub mod neutral;

#[cfg(feature = "alloc")]
pub type Term<O> = alloc::boxed::Box<dyn eval::InnerEval<Output = O>>;

#[derive(Copy, Clone, Debug)]
pub struct Identity;

impl<Value> crate::eval::UnaryEval<Value> for Identity {
    #[inline(always)]
    fn eval(&self, a: Value) -> Value {
        a
    }
}

macro_rules! unary_op {
    ($name:ident, $op:ident, $method:ident) => {
        #[derive(Copy, Clone)]
        #[cfg_attr(feature = "debug_impl", derive(Debug))]
        pub struct $name;

        impl<Value: core::ops::$op<Output = Value>> crate::eval::UnaryEval<Value> for $name {
            #[inline(always)]
            fn eval(&self, a: Value) -> Value {
                core::ops::$op::$method(a)
            }
        }
    }
}

macro_rules! binary_op {
    ($name:ident, $op:ident, $method:ident $(, $nary_neuttr:ident, $nary_neutm:ident)?) => {
        #[derive(Copy, Clone)]
        #[cfg_attr(feature = "debug_impl", derive(Debug))]
        pub struct $name;

        impl<Value: core::ops::$op<Output = Value>> crate::eval::BinaryEval<Value> for $name {
            #[inline(always)]
            fn eval(&self, a: Value, b: Value) -> Value {
                core::ops::$op::$method(a, b)
            }
        }

        $(
        impl<Value> crate::eval::NaryEval<Value> for $name
        where
            Value: core::ops::$op<Output = Value> + crate::neutral::$nary_neuttr,
        {
            fn neutral(&self) -> Value {
                crate::neutral::$nary_neuttr::$nary_neutm()
            }
        }
        )?
    }
}

pub mod logical {
    unary_op!(Not, Not, not);
    binary_op!(And, BitAnd, bitand, AllOne, allone);
    binary_op!(Or, BitOr, bitor, Zero, zero);
    binary_op!(Xor, BitXor, bitxor, Zero, zero);
    binary_op!(Shl, Shl, shl);
    binary_op!(Shr, Shr, shr);
}

pub mod numeric {
    unary_op!(Neg, Neg, neg);
    binary_op!(Add, Add, add, Zero, zero);
    binary_op!(Div, Div, div);
    binary_op!(Mul, Mul, mul, One, one);
    binary_op!(Rem, Rem, rem);
    binary_op!(Sub, Sub, sub);
}
