use num_traits::PrimInt;
use std::sync::{
    atomic::{AtomicU16, AtomicU8},
    Arc,
};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Size<I: PrimInt> {
    pub width: I,
    pub height: I,
}

impl<I: PrimInt> Size<I> {
    pub fn new(width: I, height: I) -> Self { Self { width, height } }

    pub fn area(&self) -> I { self.width * self.height }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Point<I: PrimInt> {
    pub x: I,
    pub y: I,
}

impl<I: PrimInt> Point<I> {
    pub fn new(x: I, y: I) -> Self { Self { x, y } }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Location {
    pub x: u16,
    pub y: u16,
    pub direction: u8,
}

#[derive(Debug, Clone, Default)]
pub struct AtomicLocation {
    pub x: Arc<AtomicU16>,
    pub y: Arc<AtomicU16>,
    pub direction: Arc<AtomicU8>,
}
