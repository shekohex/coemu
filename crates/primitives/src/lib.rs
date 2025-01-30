//! Primitives used by the game engine and game server.
#![cfg_attr(not(feature = "std"), no_std)]

use bytemuck::NoUninit;
use core::fmt;
use num_traits::PrimInt;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Size<I: PrimInt> {
    pub width: I,
    pub height: I,
}

impl<I: PrimInt> Size<I> {
    pub const fn new(width: I, height: I) -> Self {
        Self { width, height }
    }

    pub fn area(&self) -> I {
        self.width * self.height
    }
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
    pub fn new(x: I, y: I) -> Self {
        Self { x, y }
    }
}

impl<I: PrimInt + fmt::Display> fmt::Display for Point<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, NoUninit)]
#[repr(C, align(8))]
pub struct Location {
    /// Entity X coordinate
    pub x: u16,
    /// Entity Y coordinate
    pub y: u16,
    /// Entity direction
    pub direction: u8,

    /// Padding to align to 8 bytes
    _padding: [u8; 3],
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Location")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("direction", &self.direction)
            .finish()
    }
}

impl Location {
    pub fn new(x: u16, y: u16, direction: u8) -> Self {
        Self {
            x,
            y,
            direction,
            _padding: [0; 3],
        }
    }
}

impl From<(u16, u16, u8)> for Location {
    fn from((x, y, direction): (u16, u16, u8)) -> Self {
        Self::new(x, y, direction)
    }
}

impl From<(u16, u16)> for Location {
    fn from((x, y): (u16, u16)) -> Self {
        Self::new(x, y, 0)
    }
}

impl From<Location> for (u16, u16, u8) {
    fn from(location: Location) -> Self {
        (location.x, location.y, location.direction)
    }
}

impl From<Location> for (u16, u16) {
    fn from(location: Location) -> Self {
        (location.x, location.y)
    }
}

/// A Gauge is a value that can be incremented and decremented, but never
/// exceeds a maximum value.
///
/// Gauges are used to represent health, mana, and stamina.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, NoUninit)]
#[repr(C, align(4))]
pub struct Gauge {
    /// Current value
    pub current: u16,
    /// Maximum value
    pub max: u16,
}

impl Gauge {
    pub fn new(current: u16, max: u16) -> Self {
        Self { current, max }
    }

    pub fn full(max: u16) -> Self {
        Self { current: max, max }
    }

    pub fn current(&self) -> u16 {
        self.current
    }

    pub fn max(&self) -> u16 {
        self.max
    }

    pub fn make_full(&mut self) {
        self.current = self.max
    }

    pub fn is_full(&self) -> bool {
        self.current == self.max
    }

    pub fn is_empty(&self) -> bool {
        self.current == 0
    }

    pub fn increment(&mut self, amount: u16) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn decrement(&mut self, amount: u16) {
        self.current = self.current.saturating_sub(amount);
    }

    pub fn set(&mut self, value: u16) {
        self.current = value.min(self.max);
    }
}
