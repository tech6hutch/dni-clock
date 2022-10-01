use std::mem;

/// Used to do infallible conversions to `usize`.
///
/// The stdlib type `usize` doesn't implement conversions that don't hold true
/// on _literally every platform Rust could possibly run on_, meaning you can't
/// infallibly convert from `u32` despite `usize` more than being able to hold
/// it (on my machine, at least).
///
/// I only care a little bit about 32-bit platforms (where `u32` and `usize` are
/// equal), and I don't care at all about <32-bit, hence this trait. The static
/// assertion ensures that the crate won't even compile on platforms where
/// `usize` can't hold a `u32`.
pub trait ToUsize {
    /// Losslessly converts this type into `usize`.
    fn to_usize(self) -> usize;
}

/// Will fail on machines where pointer size is less than T.
#[allow(dead_code)] // used at compile-time
const fn assert_usize_holds<T>() {
    assert!(
        mem::size_of::<usize>() >= mem::size_of::<T>(),
        "this crate supports platforms where pointer size is at least 32 bits",
    );
}

impl ToUsize for usize {
    /// Obviously only useful for generic code
    #[inline(always)]
    fn to_usize(self) -> usize {
        self
    }
}

impl ToUsize for u32 {
    #[inline(always)]
    fn to_usize(self) -> usize {
        const _: () = assert_usize_holds::<u32>();
        self as usize
    }
}
