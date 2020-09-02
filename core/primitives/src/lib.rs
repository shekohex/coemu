use num_traits::PrimInt;

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
