//! Helper types and functions for 2D buffers.

use std::ops::{Index, IndexMut};

use crate::util::ToUsize;

/// A 2D array type but with 1D access.
///
/// 2D arrays (something like `Vec<Vec<T>>`) are easier to work with, but we
/// need access to the elements as a continuous sequence too (like `Vec<T>`).
/// This is a wrapper to give the convenience of the former while allowing a
/// zero-cost conversion to the latter.
#[derive(Clone, Default)]
pub struct Vec2d<T> {
    vec: Vec<T>,
    width: usize,
}

impl<T> Vec2d<T> {
    pub fn width(&self) -> usize {
        self.width
    }

    // todo: maybe store this as a field so it doesn't require a division
    pub fn height(&self) -> usize {
        self.vec.len().checked_div(self.width).unwrap_or(0)
    }

    /// Get a 1D view of the Vec (which is how it's stored anyway, so it's free).
    #[inline(always)]
    pub fn as_1d(&self) -> &[T] {
        &self.vec
    }

    /// Gets the row at `y` (or panics if it doesn't exist).
    pub fn row(&self, y: usize) -> &[T] {
        &self.vec[Self::row_range(self.width, y)]
    }

    /// Gets the row at `y` (or panics if it doesn't exist).
    pub fn row_mut(&mut self, y: usize) -> &mut [T] {
        &mut self.vec[Self::row_range(self.width, y)]
    }

    #[inline(always)]
    fn index_2d_to_1d<Idx: ToUsize>(width: usize, x: Idx, y: Idx) -> usize {
        y.to_usize() * width + x.to_usize()
    }

    #[inline(always)]
    fn row_range<Idx: ToUsize>(width: usize, y: Idx) -> std::ops::Range<usize> {
        let row_index = y.to_usize() * width;
        row_index..(row_index+width)
    }
}

impl<T: Copy> Vec2d<T> {
    /// Creates a `Vec2d` from a given element and size.
    ///
    /// Similar to the `vec!` macro or array expressions.
    pub fn new(value: T, width: usize, height: usize) -> Self {
        Self {
            width,
            vec: vec![value; width * height],
        }
    }

    /// Copy `src` into `self`. The top left of `src` goes into
    /// `self[(start_x, start_y)]`.
    ///
    /// Panics if `src` won't fit.
    pub fn copy_to_from(
        &mut self,
        start_x: usize,
        start_y: usize,
        src: &Self,
    ) {
        assert!(
            start_x + src.width() <= self.width() &&
            start_y + src.height() <= self.height(),
            "`src` won't fit into `self`, at least not starting from ({start_x}, {start_y})"
        );

        let self_x_range = start_x..(start_x+src.width());
        for src_y in 0..src.height() {
            let self_y = src_y + start_y;
            self.row_mut(self_y)[self_x_range.clone()].copy_from_slice(src.row(src_y));
        }
    }

    /// Copy `src` into `self`, skipping elements where
    /// `should_overwrite(current_value)` returns false. The top left of `src`
    /// goes into `self[(start_x, start_y)]`.
    ///
    /// Panics if `src` won't fit.
    pub fn copy_to_from_if(
        &mut self,
        start_x: usize,
        start_y: usize,
        src: &Self,
        mut should_overwrite: impl FnMut(T) -> bool,
    ) {
        assert!(
            start_x + src.width() <= self.width() &&
            start_y + src.height() <= self.height(),
            "`src` won't fit into `self`, at least not starting from ({start_x}, {start_y})"
        );

        for src_x in 0..src.width() {
            for src_y in 0..src.height() {
                let self_x = src_x + start_x;
                let self_y = src_y + start_y;
                let current_value = self[(self_x, self_y)];
                if should_overwrite(current_value) {
                    self[(self_x, self_y)] = src[(src_x, src_y)];
                }
            }
        }
    }
}

impl<T, Idx: ToUsize> Index<(Idx, Idx)> for Vec2d<T> {
    type Output = T;

    fn index(&self, (x, y): (Idx, Idx)) -> &Self::Output {
        &self.vec[Self::index_2d_to_1d(self.width, x, y)]
    }
}

impl<T, Idx: ToUsize> IndexMut<(Idx, Idx)> for Vec2d<T> {
    fn index_mut(&mut self, (x, y): (Idx, Idx)) -> &mut Self::Output {
        &mut self.vec[Self::index_2d_to_1d(self.width, x, y)]
    }
}
