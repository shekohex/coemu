use core::fmt;
use num_traits::PrimInt;
use std::sync::atomic::{AtomicU16, AtomicU8};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Size<I: PrimInt> {
    pub width: I,
    pub height: I,
}

impl<I: PrimInt> Size<I> {
    pub const fn new(width: I, height: I) -> Self { Self { width, height } }

    pub fn area(&self) -> I { self.width * self.height }
}

impl<I: PrimInt + fmt::Display> fmt::Display for Size<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Point<I: PrimInt> {
    pub x: I,
    pub y: I,
}

impl<I: PrimInt> Point<I> {
    pub fn new(x: I, y: I) -> Self { Self { x, y } }
}

impl<I: PrimInt + fmt::Display> fmt::Display for Point<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Location {
    pub x: u16,
    pub y: u16,
    pub direction: u8,
}

#[derive(Debug, Default)]
pub struct AtomicLocation {
    pub x: AtomicU16,
    pub y: AtomicU16,
    pub direction: AtomicU8,
}
