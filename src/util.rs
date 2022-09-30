use std::mem;

pub trait ToUsize {
    fn to_usize(self) -> usize;
}

/// Will fail on machines where pointer size is less than T.
#[allow(dead_code)] // used at compile-time
const fn assert_usize_holds<T>() {
    assert!(mem::size_of::<usize>() >= mem::size_of::<T>());
}

impl ToUsize for usize {
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
