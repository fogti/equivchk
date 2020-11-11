// bool integration

pub use num_traits::identities::One;

pub trait AllOne {
    fn allone() -> Self;
}

impl AllOne for bool {
    fn allone() -> bool {
        true
    }
}

pub trait Zero {
    fn zero() -> Self;
}

impl Zero for bool {
    fn zero() -> bool {
        false
    }
}

macro_rules! doimp {
    ($($x:ty),+) => {
        $(
        impl AllOne for $x {
            fn allone() -> $x {
                <$x>::MAX
            }
        }
        impl Zero for $x {
            fn zero() -> $x {
                0
            }
        }
        )+
    }
}

doimp!(u8, u16, u32, u64);
