pub trait InnerEval {
    type Output;
    fn eval(&self) -> Self::Output;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wrap<Value>(pub Value);
impl<Value> core::ops::Deref for Wrap<Value> {
    type Target = Value;
    #[inline(always)]
    fn deref(&self) -> &Value {
        &self.0
    }
}

impl<Value> core::ops::DerefMut for Wrap<Value> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Value {
        &mut self.0
    }
}

impl<Value: Copy> InnerEval for Wrap<Value> {
    type Output = Value;
    #[inline(always)]
    fn eval(&self) -> Value {
        self.0
    }
}

// unary evaluation

pub trait UnaryEval<Value> {
    fn eval(&self, a: Value) -> Value;
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "debug_impl", derive(Debug))]
pub struct UnaryApply<Op, Term> {
    pub op: Op,
    pub a: Term,
}

impl<Op, Term> InnerEval for UnaryApply<Op, Term>
where
    Op: UnaryEval<<Term as InnerEval>::Output>,
    Term: InnerEval,
{
    type Output = <Term as InnerEval>::Output;

    #[inline]
    fn eval(&self) -> Self::Output {
        self.op.eval(self.a.eval())
    }
}

// binary evaluation

pub trait BinaryEval<Value> {
    fn eval(&self, a: Value, b: Value) -> Value;
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "debug_impl", derive(Debug))]
pub struct BinaryApply<Op, Term> {
    pub op: Op,
    pub a: Term,
    pub b: Term,
}

impl<Op, Term> InnerEval for BinaryApply<Op, Term>
where
    Op: BinaryEval<<Term as InnerEval>::Output>,
    Term: InnerEval,
{
    type Output = <Term as InnerEval>::Output;

    #[inline]
    fn eval(&self) -> Self::Output {
        self.op.eval(self.a.eval(), self.b.eval())
    }
}

// n-ary evaluation

pub trait NaryEval<Value>: BinaryEval<Value> {
    /// returns the neutral element of the operation
    fn neutral(&self) -> Value;

    fn eval<I>(&self, a: I) -> Value
    where
        I: Iterator<Item = Value>,
    {
        // default implementation just evaluates from left to right
        a.fold(self.neutral(), |acc, i| BinaryEval::eval(self, acc, i))
    }
}

#[cfg(feature = "alloc")]
#[derive(Clone)]
#[cfg_attr(feature = "debug_impl", derive(Debug))]
pub struct NaryApply<Op, Term> {
    pub op: Op,
    pub x: alloc::vec::Vec<Term>,
}

#[cfg(feature = "alloc")]
impl<Op, Term> InnerEval for NaryApply<Op, Term>
where
    Op: NaryEval<<Term as InnerEval>::Output>,
    Term: InnerEval,
{
    type Output = <Term as InnerEval>::Output;

    #[inline]
    fn eval(&self) -> Self::Output {
        NaryEval::eval(&self.op, self.x.iter().map(InnerEval::eval))
    }
}
