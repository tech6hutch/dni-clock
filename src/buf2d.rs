//! Helper types and functions for 2D buffers.

use std::ops::{Index, IndexMut};

use crate::util::ToUsize;

#[derive(Clone, Default)]
pub struct Vec2d<T> {
    vec: Vec<T>,
    width: usize,
}

impl<T: Copy> Vec2d<T> {
    pub fn new(value: T, width: usize, height: usize) -> Self {
        Self {
            width,
            vec: vec![value; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.vec.len().checked_div(self.width).unwrap_or(0)
    }

    pub fn as_1d(&self) -> &[T] {
        &self.vec
    }

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
            "`self` must be at least as big as `src`"
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
        &self.vec[index_2d_to_1d(self.width, x, y)]
    }
}

impl<T, Idx: ToUsize> IndexMut<(Idx, Idx)> for Vec2d<T> {
    fn index_mut(&mut self, (x, y): (Idx, Idx)) -> &mut Self::Output {
        &mut self.vec[index_2d_to_1d(self.width, x, y)]
    }
}

#[inline(always)]
fn index_2d_to_1d<Idx: ToUsize>(width: usize, x: Idx, y: Idx) -> usize {
    y.to_usize() * width + x.to_usize()
}
