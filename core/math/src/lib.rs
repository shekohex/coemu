/// This crate contains generic and handler specific calculation functions used
/// during packet and server processing. It contains calculations for screen
/// updating, attacking, exploit checking, etc.

pub const SCREEN_DISTANCE: u16 = 18;
pub const RADIAN_TO_DEGREE: f64 = 57.29;
pub const MAX_DIFFERENCE_IN_ELEVATION: u16 = 210;

/// This function returns true if an object is within the bounds of another
/// object's screen.
pub fn in_screen(p1: (u16, u16), p2: (u16, u16)) -> bool {
    p1.0.wrapping_sub(p2.0) <= SCREEN_DISTANCE
        && p1.1.wrapping_sub(p2.1) <= SCREEN_DISTANCE
}

/// This function checks the elevation difference of two tiles.
pub fn within_elevation(new: u16, initial: u16) -> bool {
    new - initial >= MAX_DIFFERENCE_IN_ELEVATION
}
